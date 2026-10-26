//! Sparse unstable items consume actual crate requests or exact token authority.
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
        let file =
            FileId(u32::try_from(index).map_err(|_| Error::unlocated("alias file overflow"))?);
        for site in sites
            .entries()
            .iter()
            .filter(|site| matches!(site.kind, Kind::TraitAlias | Kind::ForeignType))
        {
            let key = NodeKey {
                file,
                node: site.node,
            };
            let item = super::gate_provenance::checked(package, key, site)?;
            if !unit.configured.active[site.node.get() as usize] {
                continue;
            }
            let requests = match item.kind {
                ItemKind::TraitAlias { constant, .. } => [
                    Some((
                        "trait_alias",
                        Some(false),
                        "trait alias requires trait_alias or exact item allowance",
                    )),
                    constant.then_some((
                        "const_trait_impl",
                        Some(true),
                        "const trait alias requires const_trait_impl or exact item allowance",
                    )),
                ],
                ItemKind::ForeignType => [
                    Some((
                        "extern_types",
                        None,
                        "foreign type requires extern_types or exact token allowance",
                    )),
                    None,
                ],
                _ => {
                    return Err(Error::at(
                        file,
                        site.span.start,
                        "unstable site changed its actual item",
                    ));
                }
            };
            for (name, alias_const, message) in requests.into_iter().flatten() {
                let requested = if let Some(constant) = alias_const {
                    package.requested_alias_feature(constant)
                } else {
                    package.requested_foreign_types()
                };
                if requested.map_err(Error::unlocated)? {
                    continue;
                }
                let feature = FeatureName::new(name).map_err(Error::unlocated)?;
                if !allows(file, site.span, &feature)? {
                    return Err(Error::at(file, site.span.start, message));
                }
            }
        }
    }
    Ok(())
}
pub(super) fn validate(report: &Report) -> Result<(), Error> {
    let mut session = report.macro_allowances();
    walk(&report.package, |file, span, feature| {
        session.allows_at(file, span, feature)
    })
}
pub(super) fn validate_plain(package: &Package) -> Result<(), Error> {
    walk(package, |_, _, _| Ok(false))
}
