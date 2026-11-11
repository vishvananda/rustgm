//! A checked declared Output premise retains its actual closure source conditions.
use super::{Clause, Declarations, Obligations, PredicateBound, RegionId};
use crate::{
    resolve::model::DefId,
    sema::closures::{actual, matching},
    types::{Kind, instance::Instance},
};

pub(super) fn append(
    facts: &mut Declarations,
    predicates: &[Clause],
    proof: &mut Obligations,
) -> Result<(), &'static str> {
    let mut lowering = None;
    for &clause in predicates {
        if !clause.has_free_regions(&facts.types)? {
            continue;
        }
        let Clause::Predicate(predicate) =
            crate::sema::traits::selection::vacuous(&mut facts.types, &mut lowering, clause)?
        else {
            continue;
        };
        if !matches!(predicate.bound, PredicateBound::ProjectionEqual(_)) {
            continue;
        }
        let Some(Kind::Projection(application)) = facts.types.kind(predicate.ty) else {
            continue;
        };
        if facts.associated.callable_output() != Some(DefId(application.item.0)) {
            continue;
        }
        let (leaf, visits) =
            crate::types::closure::reference_leaf(&facts.types, application.subject)?;
        facts.body_work.canonical_steps += visits;
        let Some(leaf) = leaf else {
            continue;
        };
        let Some(Kind::Closure { identity, .. }) = facts.types.kind(leaf) else {
            return Err("declared callable premise lost its closure leaf");
        };
        let Some(conditions) = facts.closures.conditions(identity) else {
            continue;
        };
        if conditions.is_empty() {
            continue;
        }
        let conditions = conditions.to_vec();
        facts.body_work.canonical_steps += conditions.len();
        let source = *facts
            .closures
            .identity(identity)
            .ok_or("declared callable premise has no closure source")?;
        let execution = source
            .execution
            .ok_or("declared callable premise has no checked execution")?;
        let instance = Instance::closure(&facts.types, leaf)?;
        let execution = actual::execution(facts, instance, execution)?;
        let (bindings, work) = matching::bind_instance_counted(
            &facts.types,
            execution.universals,
            execution.self_ty,
            leaf,
        )?;
        facts.body_work.canonical_steps += work;
        let actual = |region: RegionId| {
            if region == RegionId::STATIC {
                return Ok(region);
            }
            let Some((0, slot)) = region.bound_parts() else {
                return Err("declared callable condition escapes its source binder");
            };
            bindings
                .slots
                .get(&slot)
                .copied()
                .ok_or("declared callable condition has no actual source slot")
        };
        for (a, b) in conditions {
            proof.outlives.push((actual(a)?, actual(b)?));
        }
        proof.equalities.extend(bindings.equalities);
    }
    facts.body_work.canonical_steps += lowering.map_or(0, |fold| fold.steps);
    Ok(())
}
