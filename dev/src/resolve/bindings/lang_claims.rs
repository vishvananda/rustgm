//! Source language declarations consume configured targets and written formal counts once.
use super::{Declaration, reservations::error};
use crate::{
    resolve::{
        model::{DefKind, Resolution},
        package::{Error, Package},
        unit::NodeKey,
    },
    support::bindings::{DeclarationTarget, GenericRequirement, LangItem},
    syntax::{ast::Kind, item::ItemKind},
};

pub(super) fn validate(
    package: &Package,
    facts: &Resolution,
    declaration: Declaration,
    role: LangItem,
    foreign: bool,
) -> Result<(), Error> {
    let Some(profile) = role.declaration_profile() else {
        return Ok(());
    };
    let owner = item(package, declaration.origin);
    let parent = declaration
        .parent
        .and_then(|id| facts.definitions.get(id.0 as usize))
        .and_then(|parent| parent.node)
        .and_then(|node| item_kind(package, node));
    let actual = facts.definitions.get(declaration.definition.0 as usize);
    let matching = actual.is_some_and(|actual| actual.kind == kind(profile.target))
        && match profile.target {
            DeclarationTarget::Function => !foreign && parent.is_none(),
            DeclarationTarget::ForeignFunction => foreign,
            DeclarationTarget::Struct => {
                !foreign && owner.is_some_and(|item| matches!(item.kind, ItemKind::Struct { .. }))
            }
            DeclarationTarget::RequiredMethod => {
                matches!(parent, Some(ItemKind::Trait { .. }))
                    && owner.is_some_and(|item| {
                        matches!(item.kind, ItemKind::Function { body: None, .. })
                    })
            }
            DeclarationTarget::InherentMethod => {
                matches!(
                    parent,
                    Some(ItemKind::Impl {
                        trait_path: None,
                        ..
                    })
                )
            }
            DeclarationTarget::AssociatedType | DeclarationTarget::AssociatedConst => {
                matches!(parent, Some(ItemKind::Trait { .. } | ItemKind::Impl { .. }))
            }
            _ => !foreign,
        };
    if !matching {
        return Err(error(
            package,
            declaration.origin,
            "source language item differs from its required declaration target or owner",
        ));
    }
    // Foreign and variant declarations have no formal-list query in the pinned collector.
    if !foreign
        && let (Some(owner), GenericRequirement::Exact(required)) = (owner, profile.generics)
    {
        let ast = &package.unit(declaration.origin.file).configured.ast;
        let own = ast.list(owner.generics).len();
        let inherited = declaration
            .parent
            .and_then(|id| facts.definitions.get(id.0 as usize))
            .and_then(|parent| parent.node)
            .and_then(|node| {
                item(package, node).map(|item| {
                    package
                        .unit(node.file)
                        .configured
                        .ast
                        .list(item.generics)
                        .len()
                })
            })
            .unwrap_or(0);
        if own + inherited != required {
            return Err(error(
                package,
                declaration.origin,
                "source language item differs from its required generic arity",
            ));
        }
    }
    Ok(())
}
fn item(package: &Package, node: NodeKey) -> Option<&crate::syntax::item::Item> {
    let ast = &package.unit(node.file).configured.ast;
    let Kind::Item(index) = ast.node(node.node).kind else {
        return None;
    };
    ast.items.get(index as usize)
}
fn item_kind(package: &Package, node: NodeKey) -> Option<ItemKind> {
    item(package, node).map(|item| item.kind)
}
pub(crate) fn kind(target: DeclarationTarget) -> DefKind {
    match target {
        DeclarationTarget::Trait => DefKind::Trait,
        DeclarationTarget::Function
        | DeclarationTarget::ForeignFunction
        | DeclarationTarget::RequiredMethod
        | DeclarationTarget::InherentMethod => DefKind::Function,
        DeclarationTarget::Struct => DefKind::Struct,
        DeclarationTarget::Enum => DefKind::Enum,
        DeclarationTarget::Variant => DefKind::Variant,
        DeclarationTarget::AssociatedType => DefKind::TypeAlias,
        DeclarationTarget::AssociatedConst => DefKind::Const,
    }
}
