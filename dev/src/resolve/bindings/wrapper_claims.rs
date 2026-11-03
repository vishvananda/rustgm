//! Wrapper roles retain actual struct ownership and the pinned generic arity.
use super::{Declaration, reservations::error};
use crate::{
    resolve::package::{Error, Package},
    support::bindings::LangItem,
    syntax::{ast::Kind, item::ItemKind},
};
pub(super) fn validate(
    package: &Package,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), Error> {
    if !matches!(role, LangItem::ManuallyDrop | LangItem::MaybeDangling) {
        return Ok(());
    }
    let unit = package.unit(declaration.origin.file);
    let Kind::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
        return Err(error(
            package,
            declaration.origin,
            "source wrapper role requires an actual structure",
        ));
    };
    let item = unit.configured.ast.items[index as usize];
    if !matches!(item.kind, ItemKind::Struct { .. })
        || unit.configured.ast.list(item.generics).len() != 1
    {
        return Err(error(
            package,
            declaration.origin,
            "source wrapper role requires a structure with exactly one generic parameter",
        ));
    }
    Ok(())
}
