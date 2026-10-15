//! Explicit declaration and validated views share trait proofs and lifetime obligations.
mod instantiation;
mod search;
use super::assumptions::{Assumptions, Selection as Assumed};
use super::implementations::Availability;
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

/// A negative outer-head result excludes a candidate without importing its graph.
/// A positive result still requires complete equations and implementation proof.
pub(in crate::sema) fn may_have_subject(
    facts: &Declarations,
    context: &mut Context,
    item: crate::types::ItemId,
    subject: infer::TermId,
    assumptions: Option<&Assumptions>,
) -> Result<bool, infer::Error> {
    let Some(subject) = super::heads::Subject::query(&facts.types, context, subject)? else {
        return Ok(true);
    };
    Ok(
        assumptions.is_some_and(|a| a.may_have_subject(item, subject))
            || facts
                .generics
                .implementations
                .as_ref()
                .is_some_and(|registry| registry.may_have_subject(item, subject)),
    )
}

pub(in crate::sema) fn select(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
) -> Result<Selection, String> {
    select_from(facts, context, goal, caller, Availability::Validated)
}

/// Provisional declaration proofs cannot authorize executable operations. Every
/// header's contract, members and body must pass the ordinary validation pipeline.
pub(in crate::sema) fn select_declaration(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
) -> Result<Selection, String> {
    select_from(facts, context, goal, caller, Availability::Declarations)
}

fn select_from(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
    availability: Availability,
) -> Result<Selection, String> {
    match select_assumption(&facts.types, context, goal, &caller)? {
        Selection::Absent => {}
        proof => return Ok(proof),
    }
    if facts.generics.implementations.is_none() {
        return Ok(Selection::Absent);
    }
    search::prove(facts, context, goal, caller, availability)
}

/// A bound-selected method cannot replace that bound with an unrelated impl.
pub(in crate::sema) fn select_assumption(
    store: &crate::types::Store,
    context: &mut Context,
    goal: &Goal,
    caller: &Caller<'_>,
) -> Result<Selection, String> {
    match caller
        .assumptions
        .select(store, context, goal)
        .map_err(inference)?
    {
        Assumed::Proven(proof) => Ok(Selection::Proven(Evidence {
            source: Source::Assumption(proof.assumption()),
            regions: caller.first_region..caller.first_region,
            equalities: Vec::new(),
            outlives: Vec::new(),
            type_outlives: Vec::new(),
        })),
        Assumed::Deferred => Ok(Selection::Deferred),
        Assumed::Absent => Ok(Selection::Absent),
    }
}

fn inference(error: infer::Error) -> String {
    error.to_string()
}
