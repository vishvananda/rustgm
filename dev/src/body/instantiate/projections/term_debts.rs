//! Normalize scoped formation terms without converting selection debts into permissions.
use crate::{
    body::regions::{Contract, Kind},
    sema::{model::Declarations, projections},
    types::lifetime_proof::{Obligations, ScopeId},
};
use std::collections::HashMap;

pub(super) fn normalize(
    facts: &mut Declarations,
    contract: &mut Contract,
    proof: &mut Obligations,
) -> Result<(), &'static str> {
    let mut groups: Vec<(u32, Vec<usize>)> = Vec::new();
    let mut arities = HashMap::new();
    let mut work = 0_usize;
    for (position, obligation) in contract.obligations.iter().enumerate() {
        charge(&mut work, 1)?;
        if let Kind::ScopedWellFormed { ty, bound_regions } = obligation.kind
            && facts.types.contains_projections(ty) == Some(true)
        {
            let index = *arities.entry(bound_regions).or_insert_with(|| {
                groups.push((bound_regions, Vec::new()));
                groups.len() - 1
            });
            groups[index].1.push(position);
        }
    }
    for (count, positions) in groups {
        let scope = proof.scope(ScopeId::ROOT, count)?;
        let mut session = projections::concrete(proof, scope)?;
        for position in positions {
            let Kind::ScopedWellFormed { ty, .. } = &mut contract.obligations[position].kind else {
                return Err("instance formation position changed kind");
            };
            let (normalized, steps) = session.normalize(facts, *ty)?;
            charge(&mut work, steps)?;
            if facts.types.contains_projections(normalized) != Some(false) {
                return Err("concrete formation retains an associated projection");
            }
            *ty = normalized;
        }
    }
    Ok(())
}

fn charge(work: &mut usize, amount: usize) -> Result<(), &'static str> {
    *work = work.saturating_add(amount);
    if *work > 1_000_000 {
        return Err("instance clause formation work budget exceeded");
    }
    Ok(())
}
