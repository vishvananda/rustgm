//! A checked callable's declared trait premises retain their specialized lifetime witnesses.
use super::{model::Declarations, nominals::traits::State};
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
use std::collections::HashMap;

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
    universals: u32,
) -> Result<Option<Assumptions>, &'static str> {
    let mut predicates = Vec::new();
    for predicate in facts.generics.declared_predicates_in_scope(owner) {
        let body = match predicate.value {
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
            PredicateBound::Trait(_) | PredicateBound::ProjectionEqual(_)
        ) {
            predicates.push(predicate.value);
        }
    }
    let mut state = None;
    for predicate in predicates {
        let predicate = substitution.clause(&mut facts.types, predicate)?;
        if !predicate.has_free_regions(&facts.types)? {
            continue;
        }
        if predicate.contains_parameters(&facts.types)? {
            return Err("instance premise retains an unresolved parameter");
        }
        if state.is_none() {
            state = Some(State::instance(facts, universals)?);
        }
        if !state
            .as_mut()
            .ok_or("missing instance premise selection")?
            .prove_clause(facts, predicate, true)?
        {
            return Err("instance premise has no validated implementation");
        }
    }
    state
        .map(|state| witnesses(facts, state.finish(), universals))
        .transpose()
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
    let mut mapping = HashMap::new();
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
