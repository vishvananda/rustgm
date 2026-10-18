//! Normalize existing debts in their actual scopes without rewriting newly emitted evidence.
use crate::{
    body::regions::Contract,
    sema::{model::Declarations, projections},
    types::lifetime_proof::{ScopeId, TypeRoot},
};
use std::collections::HashMap;
#[cfg(test)]
#[path = "../../../../tests/invariants/resumed_projection_debts.rs"]
mod tests;

struct Group {
    scope: ScopeId,
    roots: Vec<TypeRoot>,
}

pub(super) fn normalize(
    facts: &mut Declarations,
    contract: &mut Contract,
    mut next: u32,
) -> Result<usize, &'static str> {
    let mut work = 0_usize;
    for evidence in &mut contract.proofs {
        let original = &mut evidence.proof;
        let mut groups: Vec<Group> = Vec::new();
        let mut scopes = HashMap::new();
        for root in original.type_root_positions() {
            charge(&mut work, 1)?;
            if facts.types.contains_projections(root.ty) != Some(true) {
                continue;
            }
            let index = *scopes.entry(root.scope).or_insert_with(|| {
                groups.push(Group {
                    scope: root.scope,
                    roots: Vec::new(),
                });
                groups.len() - 1
            });
            groups[index].roots.push(root);
        }
        if groups.is_empty() {
            continue;
        }
        // Other contract proofs may own the intervening identities. Resume the
        // existing binder tree while explicitly excluding those foreign ranges.
        original.resume_at(next)?;
        for group in groups {
            let replacements = {
                let mut session = projections::concrete(original, group.scope)?;
                let mut replacements = Vec::with_capacity(group.roots.len());
                for root in group.roots {
                    let (ty, steps) = session.normalize_source(facts, root.ty, root.source)?;
                    charge(&mut work, steps)?;
                    replacements.push((root, ty));
                }
                replacements
            };
            for (root, ty) in replacements {
                original.replace_type_root(root, ty)?;
            }
        }
        next = original.regions.end;
    }
    Ok(work)
}

fn charge(work: &mut usize, amount: usize) -> Result<(), &'static str> {
    *work = work.saturating_add(amount);
    if *work > 1_000_000 {
        return Err("instance projection debt work budget exceeded");
    }
    Ok(())
}
