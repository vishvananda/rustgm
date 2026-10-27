//! New role spellings retain real declaration kinds without runtime authority.
use super::{Declaration, reservations::error};
use crate::{
    resolve::package::{Error, Package},
    support::bindings::LangItem,
    syntax::{ast::Kind, item::ItemKind},
};

/// The stable attribute owns a defining declaration, distinct from weak imports.
pub(super) fn validate_handler(
    package: &Package,
    declaration: Declaration,
    foreign: bool,
) -> Result<(), Error> {
    validate(package, declaration, LangItem::PanicImpl)?;
    let ast = &package.unit(declaration.origin.file).configured.ast;
    if foreign
        || !matches!(ast.node(declaration.origin.node).kind,
            Kind::Item(index) if matches!(ast.items[index as usize].kind,
                ItemKind::Function { body: Some(_), .. }))
    {
        return Err(error(
            package,
            declaration.origin,
            "panic_handler requires its actual defining function body",
        ));
    }
    Ok(())
}

pub(super) fn validate(
    package: &Package,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), Error> {
    if role == LangItem::Ordering {
        let ast = &package.unit(declaration.origin.file).configured.ast;
        if declaration.parent.is_some()
            || !matches!(ast.node(declaration.origin.node).kind,
                Kind::Item(index) if matches!(ast.items[index as usize].kind, ItemKind::Enum { .. }))
        {
            return Err(error(
                package,
                declaration.origin,
                "source Ordering identity requires its actual module enum",
            ));
        }
        return Ok(());
    }
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
        LangItem::Not
        | LangItem::Neg
        | LangItem::Sync
        | LangItem::Unpin
        | LangItem::TrivialClone
        | LangItem::PartialEq
        | LangItem::StructuralPartialEq
        | LangItem::PartialOrd
        | LangItem::Send
        | LangItem::Sync
        | LangItem::Unpin => false,
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
            "source language operator or cloning marker requires an actual trait",
        ));
    }
    Ok(())
}
