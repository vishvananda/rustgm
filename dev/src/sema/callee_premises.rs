//! A checked callable's declared trait premises retain their specialized lifetime witnesses.
mod input_proof;
mod original_inputs;
use super::{model::Declarations, nominals::traits::State};
use crate::support::hash::HashMap;
use crate::{
    resolve::model::DefId,
    types::{
        RegionId,
        arguments::Argument,
        lifetime_proof::Obligations,
        outlives::Pair,
        parameters::{clauses::Clause, predicates::Bound as PredicateBound},
        substitute::Bound,
    },
};
pub(crate) use input_proof::Inputs;
pub(in crate::sema) use original_inputs::OriginalInputs;

pub(crate) struct Assumptions {
    pub universals: u32,
    pub regions: Vec<Pair>,
    pub types: Vec<Clause>,
}

/// Call sites have proved these exact declared predicates. A unique validated
/// impl supplies their existential witnesses in this separate body universe.
/// This is not authority for predicates discovered while normalizing its body.
pub(crate) fn specialize(
    facts: &mut Declarations,
    owner: DefId,
    substitution: &mut Bound,
    caller: Option<&crate::types::parameters::environment::Environment>,
    universals: u32,
) -> Result<Option<Assumptions>, &'static str> {
    requirements(facts, owner, substitution, caller, universals, 1)?
        .map(|proof| witnesses(facts, proof, universals))
        .transpose()
}

/// Return the selected implementation's debts before converting any of them
/// into a checked callee's declared-premise witnesses. Source callers must prove
/// this whole payload against their own actual inputs before using that route.
pub(crate) fn requirements(
    facts: &mut Declarations,
    owner: DefId,
    substitution: &mut Bound,
    caller: Option<&crate::types::parameters::environment::Environment>,
    universals: u32,
    first: u32,
) -> Result<Option<Obligations>, &'static str> {
    let mut predicates = Vec::new();
    for predicate in facts.generics.declared_predicates_in_scope(owner) {
        facts.body_work.canonical_steps += 1;
        let body = match predicate.value {
            Clause::UnstableFeature(id) => {
                let work = std::cell::Cell::new(0_usize);
                let result = facts.prove_feature(id, caller, &|steps, _| {
                    work.set(work.get().saturating_add(steps));
                    Ok(())
                });
                facts.body_work.canonical_steps =
                    facts.body_work.canonical_steps.saturating_add(work.get());
                if !result? {
                    return Err("callee feature requirement lacks actual caller evidence");
                }
                continue;
            }
            Clause::Predicate(value) => value,
            Clause::ForAll(id) => {
                facts
                    .types
                    .quantified(id)
                    .ok_or("invalid instance premise clause")?
                    .predicate
            }
        };
        if matches!(
            body.bound,
            PredicateBound::Copy
                | PredicateBound::Clone
                | PredicateBound::Sized
                | PredicateBound::Outlives(_)
        ) {
            continue;
        }
        if matches!(
            body.bound,
            PredicateBound::Trait(_)
                | PredicateBound::ConstTrait(_)
                | PredicateBound::MaybeConstTrait(_)
                | PredicateBound::ProjectionEqual(_)
        ) {
            predicates.push(crate::sema::traits::effects::in_context(
                &mut facts.types,
                predicate.value,
                crate::support::constness::Constness::Ordinary,
            )?);
        }
    }
    let predicates = predicates
        .into_iter()
        .map(|predicate| substitution.clause(&mut facts.types, predicate))
        .collect::<Result<Vec<_>, _>>()?;
    clauses(facts, &predicates, universals, first)
}

/// Only an independently validated declared implementation condition may use
/// this witness route. Its callers own the exact source or canonical contract.
fn clauses(
    facts: &mut Declarations,
    predicates: &[Clause],
    universals: u32,
    first: u32,
) -> Result<Option<Obligations>, &'static str> {
    let mut state = None;
    for &predicate in predicates {
        facts.body_work.canonical_steps += 1;
        if !predicate.has_free_regions(&facts.types)? {
            continue;
        }
        if predicate.contains_parameters(&facts.types)? {
            return Err("instance premise retains an unresolved parameter");
        }
        if state.is_none() {
            let selected = State::instance_at(facts, universals, first)?;
            facts.body_work.canonical_steps += selected.selection_steps();
            state = Some(selected);
        }
        let selected = state.as_mut().ok_or("missing instance premise selection")?;
        let before = selected.selection_steps();
        let proven = selected.prove_clause(facts, predicate, true);
        facts.body_work.canonical_steps += selected.selection_steps().saturating_sub(before);
        if !proven? {
            return Err("instance premise has no validated implementation");
        }
    }
    Ok(state.map(State::finish))
}

/// A canonical implementation condition is validated even without lifetime
/// occurrences; absence cannot authorize a factory merely because it is closed.
pub(in crate::sema) fn declared_condition(
    facts: &mut Declarations,
    predicate: Clause,
    universals: u32,
) -> Result<Assumptions, &'static str> {
    if predicate.contains_parameters(&facts.types)? {
        return Err("canonical implementation condition retains unresolved parameters");
    }
    let mut state = State::instance(facts, universals)?;
    if !state.prove_clause(facts, predicate, true)? {
        return Err("canonical implementation condition has no validated implementation");
    }
    witnesses(facts, state.finish(), universals)
}

fn witnesses(
    facts: &mut Declarations,
    proof: Obligations,
    first: u32,
) -> Result<Assumptions, &'static str> {
    if proof.has_quantified() {
        return quantified_conditions(facts, &proof, first);
    }
    let mut result = Assumptions {
        universals: first,
        regions: Vec::new(),
        types: Vec::new(),
    };
    let mut mapping = HashMap::default();
    for index in 0..first {
        let region = universal(index)?;
        mapping.insert(region, region);
    }
    for id in proof.owned_regions().flatten() {
        mapping.insert(RegionId(id), universal(result.universals)?);
        result.universals += 1;
    }
    let mut fold = Bound::restore_regions(mapping);
    for (left, right) in proof.equalities {
        let left = region(facts, &mut fold, left)?;
        let right = region(facts, &mut fold, right)?;
        if left != right {
            result.regions.extend([(left, right), (right, left)]);
        }
    }
    for (longer, shorter) in proof.outlives {
        result.regions.push((
            region(facts, &mut fold, longer)?,
            region(facts, &mut fold, shorter)?,
        ));
    }
    for (ty, shorter) in proof.types {
        result.types.push(
            crate::types::parameters::predicates::Predicate {
                ty: fold.apply(&mut facts.types, ty)?,
                bound: PredicateBound::Outlives(region(facts, &mut fold, shorter)?),
            }
            .into(),
        );
    }
    Ok(result)
}

/// A call has established the declared premise. Its selected implementation's
/// necessary outer relations therefore hold, but its forall-local witnesses
/// remain private. The directed solver eliminates them in their actual scopes.
fn quantified_conditions(
    facts: &mut Declarations,
    proof: &Obligations,
    universals: u32,
) -> Result<Assumptions, &'static str> {
    for (_, ty) in proof.type_roots() {
        facts
            .lifetimes
            .nominal_requirements
            .prepare(&mut facts.types, ty)?;
    }
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let mut constraints = crate::types::relations::Constraints::new(
        &facts.types,
        &facts.lifetimes.variances,
        universals,
        1_000_000,
    )
    .map_err(|_| "instance premise condition graph exhausted")?;
    proof
        .require(&mut constraints, &mut queries)
        .map_err(|_| "invalid instance premise lifetime conditions")?;
    let regions = constraints
        .outer_conditions()
        .map_err(|_| "instance premise cannot expose its quantified conditions")?;
    Ok(Assumptions {
        universals,
        regions,
        types: Vec::new(),
    })
}

fn universal(index: u32) -> Result<RegionId, &'static str> {
    RegionId::bound(
        0,
        u16::try_from(index).map_err(|_| "too many instance premise lifetimes")?,
    )
}

fn region(
    facts: &mut Declarations,
    fold: &mut Bound,
    original: RegionId,
) -> Result<RegionId, &'static str> {
    let Argument::Lifetime(region) =
        fold.apply_argument(&mut facts.types, Argument::Lifetime(original))?
    else {
        return Err("instance premise substitution changed lifetime kind");
    };
    Ok(region)
}

#[cfg(test)]
#[path = "../../tests/invariants/paid_selection.rs"]
mod tests;
