//! Protocol claims retain actual source owners; syntax supplies no execution.
use super::{Declaration, reservations::error};
use crate::{
    resolve::package::{Error, Package},
    support::bindings::LangItem,
    syntax::{ast::Kind, item::ItemKind},
};

#[derive(Clone, Copy)]
enum Owner {
    Trait,
    Member,
    Function,
    Variant,
}

pub(super) fn validate(
    package: &Package,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), Error> {
    let required = match role {
        LangItem::TryTrait => Owner::Trait,
        LangItem::TryFromResidual | LangItem::TryFromOutput | LangItem::TryBranch => Owner::Member,
        LangItem::IntoTryType => Owner::Function,
        LangItem::ControlFlowContinue | LangItem::ControlFlowBreak => Owner::Variant,
        _ => return Ok(()),
    };
    let ast = &package.unit(declaration.origin.file).configured.ast;
    let actual = ast.node(declaration.origin.node).kind;
    let valid = match (required, actual) {
        (Owner::Trait, Kind::Item(index)) => {
            matches!(ast.items[index as usize].kind, ItemKind::Trait { .. })
        }
        (Owner::Member | Owner::Function, Kind::Item(index)) => {
            matches!(ast.items[index as usize].kind, ItemKind::Function { .. })
                && declaration.parent.is_some() == matches!(required, Owner::Member)
        }
        (Owner::Variant, Kind::Variant(_)) => declaration.parent.is_none(),
        _ => false,
    };
    if !valid {
        return Err(error(
            package,
            declaration.origin,
            "source Try role has the wrong actual declaration kind or owner",
        ));
    }
    Ok(())
}
