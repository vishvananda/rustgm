//! Normalize existing debts in their actual scopes without rewriting newly emitted evidence.
use crate::support::hash::HashMap;
use crate::{
    body::regions::Contract,
    sema::{model::Declarations, projections},
    types::lifetime_proof::{ScopeId, TypeRoot},
};
#[cfg(test)]
#[path = "../../../../tests/invariants/resumed_projection_debts.rs"]
mod tests;

struct Group {
    scope: ScopeId,
    roots: Vec<TypeRoot>,
}

#[cfg(test)]
pub(super) fn normalize(
    facts: &mut Declarations,
    contract: &mut Contract,
    next: u32,
) -> Result<usize, &'static str> {
    normalize_with_owner(facts, contract, next, None).map(|(work, _)| work)
}

pub(super) fn normalize_with_owner(
    facts: &mut Declarations,
    contract: &mut Contract,
    mut next: u32,
    mut owner: Option<&mut super::super::allocated_owners::Operation>,
) -> Result<(usize, u32), &'static str> {
    let mut work = 0_usize;
    for (index, evidence) in contract.proofs.iter_mut().enumerate() {
        let original = &mut evidence.proof;
        let mut groups: Vec<Group> = Vec::new();
        let mut scopes = HashMap::default();
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
        let first = next;
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
        if let Some(owner) = owner.as_deref_mut() {
            owner.record(index, evidence, first..next)?;
        }
    }
    Ok((work, next))
}

fn charge(work: &mut usize, amount: usize) -> Result<(), &'static str> {
    *work = work.saturating_add(amount);
    if *work > 1_000_000 {
        return Err("instance projection debt work budget exceeded");
    }
    Ok(())
}
