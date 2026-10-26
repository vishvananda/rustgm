//! New role spellings retain real declaration kinds without runtime authority.
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
    if matches!(role, LangItem::PanicLocation | LangItem::PanicInfo) {
        let ast = &package.unit(declaration.origin.file).configured.ast;
        let Kind::Item(index) = ast.node(declaration.origin.node).kind else {
            return Err(error(
                package,
                declaration.origin,
                "source panic nominal has no actual item",
            ));
        };
        if declaration.parent.is_some()
            || !matches!(ast.items[index as usize].kind, ItemKind::Struct { .. })
        {
            return Err(error(
                package,
                declaration.origin,
                "source panic nominal requires an actual module structure",
            ));
        }
        return Ok(());
    }
    let function = match role {
        LangItem::Not => false,
        LangItem::Panic
        | LangItem::PanicFmt
        | LangItem::PanicImpl
        | LangItem::PanicDisplay
        | LangItem::ConstPanicFmt => true,
        _ => return Ok(()),
    };
    let unit = package.unit(declaration.origin.file);
    let Kind::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
        return Err(error(
            package,
            declaration.origin,
            "source panic role lost its item",
        ));
    };
    let actual = unit.configured.ast.items[index as usize].kind;
    if function {
        if !matches!(actual, ItemKind::Function { .. }) || declaration.parent.is_some() {
            return Err(error(
                package,
                declaration.origin,
                "source panic role requires an actual module function",
            ));
        }
    } else if !matches!(actual, ItemKind::Trait { .. }) {
        return Err(error(
            package,
            declaration.origin,
            "source Not role requires an actual trait",
        ));
    }
    Ok(())
}
