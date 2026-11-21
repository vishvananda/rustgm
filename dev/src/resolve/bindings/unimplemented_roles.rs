//! Bring-up recognizes alloc's `String` declaration as a struct while its
//! diagnostic role remains unsupported.
use crate::{
    resolve::{
        package::{Error, Package},
        unit::NodeKey,
    },
    syntax::{ast::Kind, item::ItemKind},
};

const STRING_WAIVER: &str = "lang item `String` is admitted as an ordinary struct: its diagnostics and the uniqueness of its role are not checked";

/// Admit `key`'s declaration of the role `name` as an ordinary struct.
/// Returns false for other item names.
/// # Errors
/// Rejects a non-struct declaration and every declaration outside bring-up.
pub(super) fn admit(
    package: &Package,
    key: NodeKey,
    offset: u32,
    name: &str,
) -> Result<bool, Error> {
    if name != "String" {
        return Ok(false);
    }
    let error = |message: String| Error::at(key.file, offset, message);
    let ast = &package.unit(key.file).configured.ast;
    let item = match ast.node(key.node).kind {
        Kind::Item(index) => ast.items.get(index as usize),
        _ => None,
    };
    if !item.is_some_and(|item| matches!(item.kind, ItemKind::Struct { .. })) {
        return Err(error(format!(
            "`{name}` lang item must be applied to a struct"
        )));
    }
    if crate::support::bringup::waive(STRING_WAIVER) {
        return Ok(true);
    }
    Err(error("unsupported source language item".into()))
}
