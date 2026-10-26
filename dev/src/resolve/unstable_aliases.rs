//! Whole-item alias spans use actual crate requests or exact expansion authority.
use super::{
    bindings::FeatureName,
    expansion::Report,
    package::{Error, Package},
    unit::{FileId, NodeKey},
};
use crate::syntax::{gate_sites::Kind, item::ItemKind, token_tree::Span};
#[cfg(test)]
#[path = "../../tests/invariants/gated_aliases.rs"]
mod tests;

fn walk(
    package: &Package,
    mut allows: impl FnMut(FileId, Span, &FeatureName<'_>) -> Result<bool, Error>,
) -> Result<(), Error> {
    for (index, unit) in package.units.iter().enumerate() {
        let Some(sites) = &unit.configured.ast.gate_sites else {
            continue;
        };
        let file = FileId(u32::try_from(index).map_err(|_| Error::unlocated("alias file overflow"))?);
        for site in sites.entries().iter().filter(|site| site.kind == Kind::TraitAlias) {
            let key = NodeKey { file, node: site.node };
            let item = super::gate_provenance::checked(package, key, site)?;
            if !unit.configured.active[site.node.get() as usize] {
                continue;
            }
            let ItemKind::TraitAlias { constant, .. } = item.kind else {
                return Err(Error::at(file, site.span.start, "alias site changed its actual item"));
            };
            for is_const in [false, true].into_iter().take(if constant { 2 } else { 1 }) {
                if package.requested_alias_feature(is_const).map_err(Error::unlocated)? {
                    continue;
                }
                let feature = FeatureName::new(if is_const { "const_trait_impl" } else { "trait_alias" })
                    .map_err(Error::unlocated)?;
                if !allows(file, site.span, &feature)? {
                    return Err(Error::at(file, site.span.start, if is_const {
                        "const trait alias requires const_trait_impl or exact item allowance"
                    } else {
                        "trait alias requires trait_alias or exact item allowance"
                    }));
                }
            }
        }
    }
    Ok(())
}
pub(super) fn validate(report: &Report) -> Result<(), Error> {
    let mut session = report.macro_allowances();
    walk(&report.package, |file, span, feature| session.allows_at(file, span, feature))
}
pub(super) fn validate_plain(package: &Package) -> Result<(), Error> {
    walk(package, |_, _, _| Ok(false))
}
