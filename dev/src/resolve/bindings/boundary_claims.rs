//! Range helper claims identify associated declarations, never endpoint semantics.
use super::{Declaration, reservations::error};
use crate::{
    resolve::{
        model::{DefKind, Resolution},
        package::{Error, Package},
    },
    support::bindings::LangItem,
    syntax::{ast::Kind, item::ItemKind},
};
pub(super) fn validate(
    package: &Package,
    facts: &Resolution,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), Error> {
    if !matches!(
        role,
        LangItem::RangeMin | LangItem::RangeMax | LangItem::RangeSub
    ) {
        return Ok(());
    }
    let failure = |message| error(package, declaration.origin, message);
    let parent = declaration
        .parent
        .and_then(|id| facts.definitions.get(id.0 as usize))
        .filter(|d| d.kind == DefKind::Trait)
        .and_then(|d| d.node)
        .ok_or_else(|| failure("source range role requires its actual trait owner"))?;
    let parent_ast = &package.unit(parent.file).configured.ast;
    if !matches!(parent_ast.node(parent.node).kind, Kind::Item(index) if matches!(parent_ast.items[index as usize].kind, ItemKind::Trait { .. }))
    {
        return Err(failure(
            "source range role has a different actual parent declaration",
        ));
    }
    let ast = &package.unit(declaration.origin.file).configured.ast;
    let Kind::Item(index) = ast.node(declaration.origin.node).kind else {
        return Err(failure("source range role has no actual item"));
    };
    let valid = matches!(
        (role, ast.items[index as usize].kind),
        (
            LangItem::RangeMin | LangItem::RangeMax,
            ItemKind::Const { .. }
        ) | (LangItem::RangeSub, ItemKind::Function { body: None, .. })
    );
    if !valid {
        return Err(failure(
            "source range role requires its actual associated constant or required trait method",
        ));
    }
    Ok(())
}
