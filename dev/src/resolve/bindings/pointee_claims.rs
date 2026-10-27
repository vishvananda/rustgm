//! Pointee declarations keep actual source kinds before semantic completion.
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
        LangItem::PointeeTrait
            | LangItem::DiscriminantKind
            | LangItem::DiscriminantType
            | LangItem::MetadataType
            | LangItem::DynMetadata
            | LangItem::PhantomData
    ) {
        return Ok(());
    }
    let unit = package.unit(declaration.origin.file);
    let Kind::Item(index) = unit.configured.ast.node(declaration.origin.node).kind else {
        return Err(error(
            package,
            declaration.origin,
            "source Pointee role has no item",
        ));
    };
    let actual = unit.configured.ast.items[index as usize].kind;
    let valid = match role {
        LangItem::PointeeTrait | LangItem::DiscriminantKind => matches!(actual, ItemKind::Trait { .. }),
        LangItem::MetadataType | LangItem::DiscriminantType => {
            declaration.parent.is_some() && matches!(actual, ItemKind::TypeAlias { .. })
        }
        LangItem::DynMetadata | LangItem::PhantomData => matches!(actual, ItemKind::Struct { .. }),
        _ => false,
    };
    if !valid {
        return Err(error(
            package,
            declaration.origin,
            "source Pointee role has the wrong declaration kind or owner",
        ));
    }
    Ok(())
}
