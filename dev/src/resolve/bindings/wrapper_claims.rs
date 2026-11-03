//! Wrapper roles retain their actual nominal kind and pinned generic requirements.
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
    if !matches!(
        role,
        LangItem::ManuallyDrop | LangItem::MaybeDangling | LangItem::MaybeUninit
    ) {
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
    let valid = if role == LangItem::MaybeUninit {
        // GenericRequirement::None imposes no arity, field or representation rule.
        matches!(item.kind, ItemKind::Union { .. })
    } else {
        matches!(item.kind, ItemKind::Struct { .. })
            && unit.configured.ast.list(item.generics).len() == 1
    };
    if !valid {
        return Err(error(
            package,
            declaration.origin,
            "source wrapper role differs from its required nominal kind or generic arity",
        ));
    }
    Ok(())
}
