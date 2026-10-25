//! Parsed keyword facts require exact feature authority before semantic admission.
use super::{
    bindings::FeatureName,
    expansion::{MacroAllowanceSession, Report},
    package::{Error, Package},
    unit::{FileId, NodeKey},
};
use crate::syntax::{
    defaultness::{Declaration, Kind},
    item::ItemKind,
};

fn parsed<'a>(
    package: &'a Package,
    key: NodeKey,
    declaration: &Declaration,
) -> Result<&'a crate::syntax::item::Item, Error> {
    let invalid = || {
        Error::at(
            key.file,
            declaration.keyword.start,
            "defaultness source owner changed",
        )
    };
    let unit = package.units.get(key.file.0 as usize).ok_or_else(invalid)?;
    unit.configured
        .ast
        .validate_defaultness_source(unit.source.original_image())
        .map_err(|error| Error::at(key.file, declaration.keyword.start, error.message))?;
    declaration
        .validate(&unit.configured.ast)
        .map_err(|error| Error::at(key.file, error.offset, error.message))?;
    if key.node != declaration.node
        || unit
            .source
            .text()
            .get(declaration.keyword.start as usize..declaration.keyword.end as usize)
            != Some("default")
    {
        return Err(invalid());
    }
    unit.configured
        .ast
        .items
        .get(declaration.item as usize)
        .ok_or_else(invalid)
}
fn context(
    package: &Package,
    file: FileId,
    declaration: &Declaration,
) -> Result<crate::expand::marks::SyntaxContextId, Error> {
    let entries = package
        .unit(file)
        .configured
        .ast
        .defaultness
        .as_ref()
        .ok_or_else(|| {
            Error::at(
                file,
                declaration.keyword.start,
                "defaultness column disappeared",
            )
        })?;
    let context = super::unit::contexts::at_counted(
        &package.units,
        file,
        declaration.keyword.start,
        |steps, links, lookups| {
            entries.charge(steps.saturating_add(links).saturating_add(lookups), 0)
        },
    )?;
    entries
        .charge(1, 0)
        .map_err(|message| Error::at(file, declaration.keyword.start, message))?;
    package
        .marks
        .outer(context)
        .map_err(|message| Error::at(file, declaration.keyword.start, message))?;
    Ok(context)
}
pub(super) fn checked<'a>(
    package: &'a Package,
    key: NodeKey,
    declaration: &Declaration,
) -> Result<&'a crate::syntax::item::Item, Error> {
    let item = parsed(package, key, declaration)?;
    if !declaration.context_matches(context(package, key.file, declaration)?.key()) {
        return Err(Error::at(
            key.file,
            declaration.keyword.start,
            "defaultness token context changed or was never sealed",
        ));
    }
    Ok(item)
}
pub(super) fn bind_pending(package: &mut Package, file: FileId) -> Result<(), Error> {
    loop {
        let Some(declaration) = package
            .unit(file)
            .configured
            .ast
            .defaultness
            .as_ref()
            .and_then(|entries| entries.pending_context())
        else {
            return Ok(());
        };
        parsed(
            package,
            NodeKey {
                file,
                node: declaration.node,
            },
            &declaration,
        )?;
        let actual = context(package, file, &declaration)?.key();
        package.units[file.0 as usize]
            .configured
            .ast
            .defaultness
            .as_mut()
            .ok_or_else(|| {
                Error::at(
                    file,
                    declaration.keyword.start,
                    "pending defaultness column disappeared",
                )
            })?
            .seal_context(declaration, actual)
            .map_err(|message| Error::at(file, declaration.keyword.start, message))?;
    }
}
fn semantic_form(
    file: FileId,
    declaration: &Declaration,
    item: &crate::syntax::item::Item,
) -> Result<(), Error> {
    let message = match declaration.kind {
        Kind::Function => return Ok(()),
        Kind::Implementation => {
            if !matches!(
                item.kind,
                ItemKind::Impl {
                    trait_path: Some(_),
                    ..
                }
            ) {
                return Err(Error::at(
                    file,
                    declaration.keyword.start,
                    "default impl requires a trait implementation",
                ));
            }
            "default impl specialization graph semantics are not implemented"
        }
        Kind::Constant | Kind::Type => {
            "default associated value specialization semantics are not implemented"
        }
    };
    Err(Error::at(file, declaration.keyword.start, message))
}
fn require(
    report: &Report,
    session: &mut MacroAllowanceSession<'_>,
    file: FileId,
    declaration: &Declaration,
) -> Result<(), Error> {
    if report
        .package
        .requested_defaultness(declaration.kind == Kind::Function)
        .map_err(Error::unlocated)?
    {
        return Ok(());
    }
    let feature = FeatureName::new("specialization").map_err(Error::unlocated)?;
    if session.allows_at(file, declaration.keyword, &feature)? {
        return Ok(());
    }
    Err(Error::at(
        file,
        declaration.keyword.start,
        "default declaration requires its specialization feature or exact token allowance",
    ))
}
fn walk(
    package: &Package,
    mut permit: impl FnMut(FileId, &Declaration) -> Result<(), Error>,
) -> Result<(), Error> {
    for (file, unit) in package.units.iter().enumerate() {
        let Some(entries) = &unit.configured.ast.defaultness else {
            continue;
        };
        let file = FileId(
            u32::try_from(file)
                .map_err(|_| Error::unlocated("defaultness file identity overflow"))?,
        );
        for declaration in entries.entries() {
            let key = NodeKey {
                file,
                node: declaration.node,
            };
            let item = checked(package, key, declaration)?;
            if !unit.configured.active[declaration.node.get() as usize] {
                continue;
            }
            permit(file, declaration)?;
            semantic_form(file, declaration, item)?;
        }
    }
    Ok(())
}
pub(super) fn validate(report: &Report) -> Result<(), Error> {
    let mut session = report.macro_allowances();
    walk(&report.package, |file, declaration| {
        require(report, &mut session, file, declaration)
    })
}
pub(super) fn validate_plain(package: &Package) -> Result<(), Error> {
    walk(package, |file, declaration| {
        if package
            .requested_defaultness(declaration.kind == Kind::Function)
            .map_err(Error::unlocated)?
        {
            return Ok(());
        }
        Err(Error::at(
            file,
            declaration.keyword.start,
            "default declaration requires its specialization feature",
        ))
    })
}
pub(super) fn prepare_plain(package: &mut Package) -> Result<(), Error> {
    for index in 0..package.units.len() {
        let file = FileId(
            u32::try_from(index)
                .map_err(|_| Error::unlocated("defaultness file identity overflow"))?,
        );
        bind_pending(package, file)?;
    }
    Ok(())
}

impl Report {
    /// A local active header's parsed modifier, authenticated before semantic use.
    pub(crate) fn active_defaultness(&self, key: NodeKey) -> Result<Option<Declaration>, Error> {
        let invalid = || {
            Error::at(
                key.file,
                0,
                "defaultness query requires its actual active local item",
            )
        };
        let unit = self
            .package
            .units
            .get(key.file.0 as usize)
            .ok_or_else(invalid)?;
        let node = unit
            .configured
            .ast
            .nodes
            .get(key.node.get() as usize)
            .ok_or_else(invalid)?;
        let crate::syntax::ast::Kind::Item(index) = node.kind else {
            return Err(invalid());
        };
        if unit.configured.active.get(key.node.get() as usize) != Some(&true) {
            return Err(invalid());
        }
        if let Some(entries) = &unit.configured.ast.defaultness {
            entries
                .charge(1 + entries.len().checked_ilog2().unwrap_or(0) as usize, 0)
                .map_err(|message| Error::at(key.file, node.span.start, message))?;
        }
        let Some(declaration) = unit.configured.ast.defaultness(index).copied() else {
            return Ok(None);
        };
        checked(&self.package, key, &declaration)?;
        Ok(Some(declaration))
    }
}
