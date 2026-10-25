//! Compiler crate options retain checked root ownership, not operation authority.
use crate::{
    resolve::{
        bindings::FeatureName,
        expansion::Report,
        package::Error,
        unit::{FileId, NodeKey, Unit},
    },
    syntax::{
        attribute::{Attribute, Input, Style},
        token_tree::Forest,
    },
};

#[derive(Clone, Copy)]
enum OptionKind {
    Coherence,
    PreserveUb,
}
impl OptionKind {
    fn named(name: &str) -> Option<Self> {
        match name {
            "rustc_coherence_is_core" => Some(Self::Coherence),
            "rustc_preserve_ub_checks" => Some(Self::PreserveUb),
            _ => None,
        }
    }
    const fn mask(self) -> u8 {
        match self {
            Self::Coherence => 1 << 3,
            Self::PreserveUb => 1 << 4,
        }
    }
}
/// Only this parsed-root producer can construct the option bits.
#[derive(Clone, Copy, Default)]
pub(super) struct Options(u8);
impl Options {
    pub(super) const fn bits(self) -> u8 {
        self.0
    }
    fn insert(&mut self, kind: OptionKind, file: FileId, offset: u32) -> Result<(), Error> {
        if self.0 & kind.mask() != 0 {
            return Err(Error::at(
                file,
                offset,
                "duplicate compiler crate option is not supported",
            ));
        }
        self.0 |= kind.mask();
        Ok(())
    }
}
pub(super) fn prepare(unit: &Unit, internal: bool) -> Result<Options, Error> {
    let root = unit.configured.ast.root;
    let Some(index) = unit.owner_indices[root.get() as usize] else {
        return Ok(Options::default());
    };
    let owner = &unit.configured.owners[index.get() as usize - 1];
    if !owner.enabled || owner.node != root {
        return Ok(Options::default());
    }
    let mut options = Options::default();
    for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
        let Some(kind) = attribute
            .name(forest, unit.source.text())
            .and_then(OptionKind::named)
        else {
            continue;
        };
        form(attribute, FileId(0))?;
        if !internal {
            return Err(Error::at(
                FileId(0),
                attribute.span.start,
                "compiler crate option requires rustc_attrs",
            ));
        }
        options.insert(kind, FileId(0), attribute.span.start)?;
    }
    Ok(options)
}

/// Validate actual published ownership after expansion, including later modules.
pub(in crate::resolve) fn validate(report: &Report) -> Result<(), Error> {
    validate_requests(report)?;
    let root = report.package.modules.first().ok_or_else(|| {
        Error::at(
            FileId(0),
            0,
            "compiler crate options have no actual crate root",
        )
    })?;
    let expected = NodeKey {
        file: root.file,
        node: root.root,
    };
    let mut seen = Options::default();
    let mut allowances = report.macro_allowances();
    for (index, unit) in report.package.units.iter().enumerate() {
        // Include templates retain provenance; installed owners are validated once.
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(index)
                .map_err(|_| Error::at(FileId(0), 0, "too many compiler option files"))?,
        );
        for owner in unit.configured.owners.iter().filter(|owner| owner.enabled) {
            for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
                let Some(kind) = attribute
                    .name(forest, unit.source.text())
                    .and_then(OptionKind::named)
                else {
                    continue;
                };
                let error = |message| Error::at(file, attribute.span.start, message);
                if let Some(source) = report.package.source_core() {
                    source
                        .macros
                        .charge(
                            1 + attribute.path.end.saturating_sub(attribute.path.start) as usize,
                            0,
                        )
                        .map_err(error)?;
                }
                form(attribute, file)?;
                let key = NodeKey {
                    file,
                    node: owner.node,
                };
                if key != expected {
                    return Err(error(
                        "compiler crate option requires the actual crate root owner",
                    ));
                }
                require_internal(report, &mut allowances, file, attribute, forest)?;
                seen.insert(kind, file, attribute.span.start)?;
            }
        }
    }
    let actual = u8::from(report.package.coherence_is_core()) << 3
        | u8::from(report.package.preserves_ub_checks()) << 4;
    if seen.bits() != actual {
        return Err(Error::at(
            expected.file,
            0,
            "compiler crate options changed after root preparation",
        ));
    }
    Ok(())
}
fn validate_requests(report: &Report) -> Result<(), Error> {
    if let Some(features) = report.package.source_features() {
        let requested = report.package.requested_cfg_ub_checks()
            == features.has(super::policy::Feature::CfgUbChecks);
        if !requested {
            return Err(Error::at(
                FileId(0),
                0,
                "compiler crate feature request snapshot changed",
            ));
        }
    }
    Ok(())
}
fn form(attribute: Attribute, file: FileId) -> Result<(), Error> {
    if attribute.style != Style::Inner
        || attribute.input != Input::Empty
        || attribute.unsafe_wrapper
    {
        return Err(Error::at(
            file,
            attribute.span.start,
            "compiler crate option requires an inner word attribute without arguments",
        ));
    }
    Ok(())
}
fn require_internal(
    report: &Report,
    allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
    file: FileId,
    attribute: Attribute,
    forest: &Forest,
) -> Result<(), Error> {
    if report
        .package
        .macro_features
        .require_internal(file, attribute.span.start)
        .is_ok()
    {
        return Ok(());
    }
    let name = forest
        .tree(attribute.path.start)
        .ok_or_else(|| {
            Error::at(
                file,
                attribute.span.start,
                "compiler crate option has no actual name token",
            )
        })?
        .span;
    let feature =
        FeatureName::new("rustc_attrs").map_err(|message| Error::at(file, name.start, message))?;
    if !allowances.allows(
        crate::expand::stream::Location { file, span: name },
        &feature,
    )? {
        return Err(Error::at(
            file,
            name.start,
            "compiler crate option requires rustc_attrs",
        ));
    }
    Ok(())
}
