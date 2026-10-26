//! Parsed keyword facts require exact feature authority before semantic admission.
use super::{
    bindings::FeatureName,
    expansion::{MacroAllowanceSession, Report},
    package::{Error, Package},
    unit::{FileId, NodeKey},
};
use crate::syntax::{
    gate_sites::{Declaration, Kind},
    item::ItemKind,
};

pub(super) use super::gate_provenance::{bind_pending, checked, prepare_plain};
fn semantic_form(
    file: FileId,
    declaration: &Declaration,
    item: &crate::syntax::item::Item,
) -> Result<(), Error> {
    let message = match declaration.kind {
        Kind::Function => return Ok(()),
        Kind::TraitAlias | Kind::ForeignType | Kind::FinalFunction => {
            return Err(Error::at(
                file,
                declaration.span.start,
                "feature site is not a default modifier",
            ));
        }
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
                    declaration.span.start,
                    "default impl requires a trait implementation",
                ));
            }
            "default impl specialization graph semantics are not implemented"
        }
        Kind::Constant | Kind::Type => {
            "default associated value specialization semantics are not implemented"
        }
    };
    Err(Error::at(file, declaration.span.start, message))
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
    if session.allows_at(file, declaration.span, &feature)? {
        return Ok(());
    }
    Err(Error::at(
        file,
        declaration.span.start,
        "default declaration requires its specialization feature or exact token allowance",
    ))
}
fn walk(
    package: &Package,
    mut permit: impl FnMut(FileId, &Declaration) -> Result<(), Error>,
) -> Result<(), Error> {
    for (file, unit) in package.units.iter().enumerate() {
        if !unit.configured.ast.has_defaultness() {
            continue;
        }
        let Some(entries) = &unit.configured.ast.gate_sites else {
            continue;
        };
        let file = FileId(
            u32::try_from(file)
                .map_err(|_| Error::unlocated("defaultness file identity overflow"))?,
        );
        for declaration in entries
            .entries()
            .iter()
            .filter(|site| site.kind.is_defaultness())
        {
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
            declaration.span.start,
            "default declaration requires its specialization feature",
        ))
    })
}

impl Report {
    /// A local active header's parsed modifier, authenticated before semantic use.
    pub(crate) fn active_defaultness(&self, key: NodeKey) -> Result<Option<Declaration>, Error> {
        Ok(self
            .active_gate_site(key)?
            .filter(|site| site.kind.is_defaultness()))
    }
    pub(crate) fn active_finalness(&self, key: NodeKey) -> Result<Option<Declaration>, Error> {
        Ok(self
            .active_gate_site(key)?
            .filter(|site| site.kind == Kind::FinalFunction))
    }
    pub(crate) fn active_method_modifier(
        &self,
        key: NodeKey,
    ) -> Result<Option<Declaration>, Error> {
        Ok(self
            .active_gate_site(key)?
            .filter(|site| site.kind.is_defaultness() || site.kind == Kind::FinalFunction))
    }
    fn active_gate_site(&self, key: NodeKey) -> Result<Option<Declaration>, Error> {
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
        if let Some(entries) = &unit.configured.ast.gate_sites {
            entries
                .charge(1 + entries.len().checked_ilog2().unwrap_or(0) as usize, 0)
                .map_err(|message| Error::at(key.file, node.span.start, message))?;
        }
        let Some(declaration) = unit.configured.ast.gate_site(index).copied() else {
            return Ok(None);
        };
        checked(&self.package, key, &declaration)?;
        Ok(Some(declaration))
    }
}
