//! Direct const declarations and actual const trait/impl containers retain identity.
use super::{Draft, Error, Package, Resolution};
use crate::{
    resolve::model::{DefId, DefKind},
    support::hash::HashSet,
    syntax::{ast::Kind, item::ItemKind},
};

pub(super) fn collect(
    package: &Package,
    facts: &Resolution,
    draft: &mut Draft<'_>,
) -> Result<HashSet<DefId>, Error> {
    draft.charge(
        facts.definitions.len().saturating_mul(2),
        facts.definitions.len().saturating_mul(64),
    )?;
    let mut containers = HashSet::default();
    for definition in &facts.definitions {
        let Some(key) = definition.node else {
            continue;
        };
        let unit = package.unit(key.file);
        let Kind::Item(index) = unit.configured.ast.node(key.node).kind else {
            continue;
        };
        if matches!(
            unit.configured.ast.items[index as usize].kind,
            ItemKind::Trait { constant: true, .. } | ItemKind::Impl { constant: true, .. }
        ) {
            let module = facts.scopes[definition.scope.0 as usize].module;
            if let Some(scope) = facts.node_scopes.get(&(module, key.node)) {
                containers.insert(*scope);
            }
        }
    }
    let mut result = HashSet::default();
    for (index, definition) in facts.definitions.iter().enumerate() {
        if definition.kind != DefKind::Function {
            continue;
        }
        let Some(key) = definition.node else {
            continue;
        };
        let unit = package.unit(key.file);
        let Kind::Item(item) = unit.configured.ast.node(key.node).kind else {
            continue;
        };
        if let ItemKind::Function {
            flags,
            body: Some(_),
            ..
        } = unit.configured.ast.items[item as usize].kind
            && (flags.constant || draft.is_comptime(key)? || containers.contains(&definition.scope))
            && !draft.has_marker(key, super::super::FunctionMarker::NonConstMember)?
        {
            result
                .insert(DefId(u32::try_from(index).map_err(|_| {
                    Error::unlocated("const target identity overflow")
                })?));
        }
    }
    Ok(result)
}
