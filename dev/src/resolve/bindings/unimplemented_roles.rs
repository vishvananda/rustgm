//! Lang items that rustc 1.98.0 defines on a struct and alloc declares, but
//! that this compiler's vocabulary gives no role. `global_alloc_ty` lets
//! layout and calling conventions treat a `Box` over the global allocator as
//! a plain pointer. `String` only feeds rustc's diagnostics and lints. None
//! of these meanings exists here, so a
//! bring-up session admits each declaration as an ordinary struct with a
//! recorded waiver, once it meets rustc's target and generic requirement,
//! and a strict session still rejects it as unsupported.
use crate::{
    resolve::{
        package::{Error, Package},
        unit::NodeKey,
    },
    syntax::{ast::Kind, item::ItemKind},
};

/// One role: its spelling, rustc's minimum generic parameter count, and the
/// waiver that admits it.
struct Role {
    name: &'static str,
    minimum: usize,
    waiver: &'static str,
}
const ROLES: [Role; 2] = [
    Role {
        name: "global_alloc_ty",
        minimum: 0,
        waiver: "lang item `global_alloc_ty` is admitted as an ordinary struct: the global allocator's Box layout and calling convention are not implemented",
    },
    Role {
        name: "String",
        minimum: 0,
        waiver: "lang item `String` is admitted as an ordinary struct: its diagnostics and the uniqueness of its role are not checked",
    },
];

/// Admit `key`'s declaration of the role `name` as an ordinary struct.
/// Returns false when `name` is none of these roles.
/// # Errors
/// Rejects a declaration that is not a struct or has fewer generic
/// parameters than rustc requires, and every declaration outside bring-up.
pub(super) fn admit(
    package: &Package,
    key: NodeKey,
    offset: u32,
    name: &str,
) -> Result<bool, Error> {
    let Some(role) = ROLES.iter().find(|role| role.name == name) else {
        return Ok(false);
    };
    let error = |message: String| Error::at(key.file, offset, message);
    let ast = &package.unit(key.file).configured.ast;
    let item = match ast.node(key.node).kind {
        Kind::Item(index) => ast.items.get(index as usize),
        _ => None,
    };
    let Some(item) = item.filter(|item| matches!(item.kind, ItemKind::Struct { .. })) else {
        return Err(error(format!(
            "`{name}` lang item must be applied to a struct"
        )));
    };
    if ast.list(item.generics).len() < role.minimum {
        return Err(error(format!(
            "`{name}` lang item must be applied to a struct with at least {} generic argument",
            role.minimum
        )));
    }
    if crate::support::bringup::waive(role.waiver) {
        return Ok(true);
    }
    Err(error("unsupported source language item".into()))
}
