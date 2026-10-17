//! A checked callable's declared trait premises retain their specialized lifetime witnesses.
use super::{model::Declarations, nominals::traits::State};
use crate::{
    resolve::model::DefId,
    types::{
        RegionId, TypeId, arguments::Argument, lifetime_proof::Obligations, outlives::Pair,
        parameters::predicates::Bound as PredicateBound, substitute::Bound,
    },
};
use std::collections::HashMap;

pub(crate) struct Assumptions {
    pub universals: u32,
    pub regions: Vec<Pair>,
    pub types: Vec<(TypeId, RegionId)>,
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
    let predicates = facts
        .generics
        .predicates_in_scope(owner)
        .filter(|p| matches!(p.value.bound, PredicateBound::Trait(_)))
        .map(|p| p.value)
        .collect::<Vec<_>>();
    let mut state = None;
    for predicate in predicates {
        let predicate = substitution.predicate(&mut facts.types, predicate)?;
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
            .prove(facts, predicate, true)?
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
        return Err("instance premise witnesses require their quantified scope");
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
    for id in proof.regions.clone() {
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
        result.types.push((
            fold.apply(&mut facts.types, ty)?,
            region(facts, &mut fold, shorter)?,
        ));
    }
    Ok(result)
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
