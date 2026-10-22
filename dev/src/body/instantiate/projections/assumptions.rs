//! Concrete caller premises retain their binder and every normalization requirement.
use crate::support::hash::HashMap;
use crate::{
    body::regions::Contract,
    sema::{model::Declarations, projections},
    types::{
        Kind, RegionId, TypeId,
        lifetime_proof::{Obligations, ScopeId},
        outlives::TypeComponents,
        parameters::{
            clauses::Clause,
            predicates::{Bound, Predicate},
        },
    },
};

struct Group {
    count: u32,
    roots: Vec<(TypeId, RegionId)>,
}

pub(super) fn normalize(
    facts: &mut Declarations,
    contract: &mut Contract,
    proof: &mut Obligations,
) -> Result<usize, &'static str> {
    let mut groups: Vec<Group> = Vec::new();
    let mut arities = HashMap::default();
    let mut work = 0;
    for &clause in &contract.type_assumptions {
        charge(&mut work, 1)?;
        let Clause::ForAll(id) = clause else {
            continue;
        };
        let value = facts.types.quantified(id).ok_or("missing caller clause")?;
        let Bound::Outlives(shorter) = value.predicate.bound else {
            return Err("body lifetime premise is not an outlives clause");
        };
        if facts.types.contains_projections(value.predicate.ty) != Some(true) {
            continue;
        }
        let index = *arities.entry(value.bound_regions).or_insert_with(|| {
            groups.push(Group {
                count: value.bound_regions,
                roots: Vec::new(),
            });
            groups.len() - 1
        });
        groups[index].roots.push((value.predicate.ty, shorter));
    }
    let mut targets = HashMap::default();
    let mut components: Vec<(RegionId, Vec<TypeId>)> = Vec::new();
    for group in groups {
        let scope = proof.scope(ScopeId::ROOT, group.count)?;
        let mut session = projections::concrete(proof, scope)?;
        for (root, shorter) in group.roots {
            let (ty, steps) = session.normalize(facts, root)?;
            charge(&mut work, steps)?;
            let shorter = match shorter.bound_parts() {
                Some((0, _)) => RegionId::STATIC,
                Some((depth, slot)) => RegionId::bound(depth - 1, slot)?,
                None => shorter,
            };
            let index = *targets.entry(shorter).or_insert_with(|| {
                components.push((shorter, Vec::new()));
                components.len() - 1
            });
            components[index].1.push(ty);
        }
    }
    materialize(facts, contract, components, &mut work)?;
    Ok(work)
}

/// Only an actual declared caller premise supplies these independent bounds.
/// Normalization debts remain in `proof`; none of them becomes a new assumption.
/// The raw projected clause remains available for its original application too.
fn materialize(
    facts: &mut Declarations,
    contract: &mut Contract,
    groups: Vec<(RegionId, Vec<TypeId>)>,
    work: &mut usize,
) -> Result<(), &'static str> {
    let mut query = TypeComponents::default();
    for (shorter, roots) in groups {
        let before = query.steps();
        let (regions, parameters) = query.clause_roots(&facts.types, &roots)?;
        charge(work, query.steps().saturating_sub(before))?;
        contract
            .assumptions
            .extend(regions.into_iter().map(|longer| (longer, shorter)));
        for id in parameters {
            contract.type_assumptions.push(
                Predicate {
                    ty: facts.types.intern(Kind::Parameter(id))?,
                    bound: Bound::Outlives(shorter),
                }
                .into(),
            );
        }
    }
    Ok(())
}

fn charge(work: &mut usize, amount: usize) -> Result<(), &'static str> {
    *work = work.saturating_add(amount);
    if *work > 1_000_000 {
        return Err("instance caller premise work budget exceeded");
    }
    Ok(())
}
