//! Complete trait proofs select validated impls and retain their lifetime obligations.
mod instantiation;
mod search;
use super::assumptions::{Assumptions, Selection as Assumed};
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        GenericArgs, RegionId, TypeId,
        infer::{self, Context, predicates::Predicate as Goal},
        outlives::Pair,
        parameters::{environment::Environment, predicates::Predicate},
    },
};
use std::{ops::Range, sync::Arc};

#[derive(Clone, Copy)]
pub(in crate::sema) enum Source {
    Assumption(Predicate),
    Implementation {
        owner: DefId,
        arguments: GenericArgs,
    },
}
pub(in crate::sema) struct Evidence {
    pub source: Source,
    pub regions: Range<u32>,
    pub equalities: Vec<Pair>,
    pub outlives: Vec<Pair>,
    pub type_outlives: Vec<(TypeId, RegionId)>,
}
pub(in crate::sema) enum Selection {
    Absent,
    Deferred,
    Proven(Evidence),
}
pub(in crate::sema) struct Caller<'a> {
    pub assumptions: &'a Assumptions,
    pub environment: Option<&'a Arc<Environment>>,
    /// First unused free region identity in the caller's eventual proof graph.
    pub first_region: u32,
}

pub(in crate::sema) fn select(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
) -> Result<Selection, String> {
    match caller
        .assumptions
        .select(&facts.types, context, goal)
        .map_err(inference)?
    {
        Assumed::Proven(proof) => {
            return Ok(Selection::Proven(Evidence {
                source: Source::Assumption(proof.assumption()),
                regions: caller.first_region..caller.first_region,
                equalities: Vec::new(),
                outlives: Vec::new(),
                type_outlives: Vec::new(),
            }));
        }
        Assumed::Deferred => return Ok(Selection::Deferred),
        Assumed::Absent => {}
    }
    if facts.generics.implementations.is_none() {
        return Ok(Selection::Absent);
    }
    search::prove(facts, context, goal, caller)
}

fn inference(error: infer::Error) -> String {
    error.to_string()
}
