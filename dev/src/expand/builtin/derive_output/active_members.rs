//! One bounded active field inventory is shared by every selected derive member.
use super::{Charge, ItemKind, Kind, LexError, NodeId, Site, error};
use crate::syntax::ast::List;
fn append(
    site: &Site<'_>,
    range: List,
    output: &mut Vec<NodeId>,
    charge: &Charge<'_>,
) -> Result<(), LexError> {
    for &node in site.ast.list(range) {
        charge(1, 0)?;
        if site.active.get(node.get() as usize) != Some(&true) {
            continue;
        }
        let growth =
            crate::support::retained::Growth::vector::<NodeId>(output.len(), output.capacity(), 1);
        charge(
            growth.work,
            growth.bytes.saturating_add(size_of::<Vec<NodeId>>()),
        )?;
        output.try_reserve(1).map_err(|_| {
            error(
                site.original.start,
                "derived field inventory allocation failed",
            )
        })?;
        output.push(node);
    }
    Ok(())
}
pub(super) fn collect(site: &Site<'_>, charge: &Charge<'_>) -> Result<Vec<NodeId>, LexError> {
    charge(1, 0)?;
    let mut output = Vec::new();
    match site.item.kind {
        ItemKind::Struct { fields, .. } => append(site, fields, &mut output, charge)?,
        ItemKind::Enum { variants } => {
            for &node in site.ast.list(variants) {
                charge(1, 0)?;
                if site.active.get(node.get() as usize) != Some(&true) {
                    continue;
                }
                let mut inner = node;
                while let Kind::Attributed { value, .. } = site.ast.node(inner).kind {
                    charge(1, 0)?;
                    inner = value;
                }
                let Kind::Variant(index) = site.ast.node(inner).kind else {
                    return Err(error(
                        site.original.start,
                        "derived enum lost its actual variant",
                    ));
                };
                append(
                    site,
                    site.ast.variants[index as usize].fields,
                    &mut output,
                    charge,
                )?;
            }
        }
        _ => {
            return Err(error(
                site.original.start,
                "this source derive requires a struct or enum",
            ));
        }
    }
    Ok(output)
}
