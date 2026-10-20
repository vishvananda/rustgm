//! Explicit declaration and validated views share trait proofs and lifetime obligations.
mod header_inputs;
mod instantiation;
mod known_premises;
mod readiness;
mod result_templates;
mod search;
use super::assumptions::{Assumptions, ScopedEvidence, ScopedSelection as Assumed};
use super::implementations::Availability;
use super::obligations::Obligations;
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{
        GenericArgs, RegionId, TypeId,
        infer::{self, Context, predicates::Predicate as Goal},
        parameters::{
            clauses::{Clause, QuantifiedId},
            environment::Environment,
            predicates::Predicate,
        },
    },
};
pub(in crate::sema) use known_premises::normalized as known_normalized;
#[cfg(test)]
pub(in crate::sema) use known_premises::opaque_normalized;
pub(in crate::sema) use readiness::{has_known_subject, requires_resolved_inputs};
pub(in crate::sema) use result_templates::symbolic_result;
pub(in crate::sema) use search::{
    Pending, Progress, begin, begin_normalized, begin_projection, begin_refining,
    begin_scoped_refining,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema) enum Dependency {
    Header(DefId),
    Fields(crate::types::ItemId),
}
impl Dependency {
    fn ready(self, facts: &Declarations) -> bool {
        match self {
            Self::Header(owner) => facts
                .generics
                .implementations
                .as_ref()
                .is_some_and(|registry| registry.header(owner).is_some()),
            Self::Fields(item) => facts.aggregates.get(item).is_some(),
        }
    }
}

#[derive(Clone, Copy)]
pub(in crate::sema) enum Source {
    Equality,
    /// A language-provided bound, never a trait method or implementation target.
    Builtin,
    Assumption(Predicate),
    /// Original caller clause and this use's complete instantiated application.
    QuantifiedAssumption {
        clause: QuantifiedId,
        subject: TypeId,
        application: crate::types::traits::ApplicationId,
    },
    /// A declared result bound, with its parent projection proof retained.
    ResultBound(Predicate),
    /// A quantified declared result, authorized only by its retained parent proof.
    QuantifiedResultBound {
        clause: QuantifiedId,
        subject: TypeId,
        application: crate::types::traits::ApplicationId,
    },
    Implementation {
        owner: DefId,
        arguments: GenericArgs,
    },
}
impl Source {
    fn result(evidence: &ScopedEvidence) -> Result<Self, infer::Error> {
        Ok(match Self::assumed(evidence)? {
            Self::Assumption(value) => Self::ResultBound(value),
            Self::QuantifiedAssumption {
                clause,
                subject,
                application,
            } => Self::QuantifiedResultBound {
                clause,
                subject,
                application,
            },
            _ => return Err(infer::Error::InvalidIdentity),
        })
    }
    fn assumed(evidence: &ScopedEvidence) -> Result<Self, infer::Error> {
        Ok(match evidence.clause {
            Clause::Predicate(predicate) => Self::Assumption(predicate),
            Clause::ForAll(clause) => {
                let crate::types::parameters::predicates::Bound::Trait(application) =
                    evidence.predicate.bound
                else {
                    return Err(infer::Error::InvalidIdentity);
                };
                Self::QuantifiedAssumption {
                    clause,
                    subject: evidence.predicate.ty,
                    application,
                }
            }
        })
    }
}
pub(in crate::sema) struct Evidence {
    /// At least one selected branch assumed an active nominal's unfinished size.
    pub provisional_size: bool,
    pub source: Source,
    pub proof: Obligations,
    pub sized: Vec<TypeId>,
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
    /// Alias normalization retains builtin premises whose implicit parameter bounds are unenforced.
    pub retain_sized: bool,
}
pub(in crate::sema) struct OwnedCaller {
    pub assumptions: Arc<Assumptions>,
    pub environment: Option<Arc<Environment>>,
    pub first_region: u32,
    pub retain_sized: bool,
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
    let subject = match super::heads::Subject::query(&facts.types, context, subject) {
        Ok(subject) => subject,
        Err(infer::Error::Normalization) => return Ok(true),
        Err(error) => return Err(error),
    };
    let Some(subject) = subject else {
        return Ok(true);
    };
    if let super::heads::Subject::Rigid(super::heads::Head::Opaque(id)) = subject {
        return Ok(super::results::may_have_opaque_trait(facts, id, item));
    }
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
    select_from(facts, context, goal, caller, Availability::Validated, false)
}

/// Provisional declaration proofs cannot authorize executable operations. Every
/// header's contract, members and body must pass the ordinary validation pipeline.
pub(in crate::sema) fn select_declaration(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
) -> Result<Selection, String> {
    select_from(
        facts,
        context,
        goal,
        caller,
        Availability::Declarations,
        false,
    )
}

pub(in crate::sema) fn select_available(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
    availability: Availability,
) -> Result<Selection, String> {
    select_from(facts, context, goal, caller, availability, false)
}

/// The requesting normalizer retains every input's formation and lifetime proof.
/// Only this root uses that authority; newly discovered impl conditions still
/// normalize independently within the search's immutable caller scope.
pub(in crate::sema) fn select_normalized(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
    availability: Availability,
) -> Result<Selection, String> {
    select_from(facts, context, goal, caller, availability, true)
}

fn select_from(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &Goal,
    caller: Caller<'_>,
    availability: Availability,
    normalized_inputs: bool,
) -> Result<Selection, String> {
    if goal.projection_result().is_some() || goal.builtin_bound().is_some() {
        // Builtin permissions use the caller's ordinary ability environment;
        // they cannot enter the user-trait assumption index.
        return search::prove(
            facts,
            context,
            goal,
            caller,
            availability,
            normalized_inputs && goal.projection_result().is_none(),
        );
    }
    match select_assumption(&mut facts.types, context, goal, &caller) {
        Ok(Selection::Absent) => {}
        Ok(proof) => return Ok(proof),
        Err(infer::Error::Normalization) => {
            return search::prove(
                facts,
                context,
                goal,
                caller,
                availability,
                normalized_inputs,
            );
        }
        Err(error) => return Err(inference(error)),
    }
    search::prove(
        facts,
        context,
        goal,
        caller,
        availability,
        normalized_inputs,
    )
}

/// A bound-selected method cannot replace that bound with an unrelated impl.
pub(in crate::sema) fn select_assumption(
    store: &mut crate::types::Store,
    context: &mut Context,
    goal: &Goal,
    caller: &Caller<'_>,
) -> Result<Selection, infer::Error> {
    match caller
        .assumptions
        .select_scoped(store, context, goal, caller.first_region)?
    {
        Assumed::Proven(proof) => {
            let source = Source::assumed(&proof)?;
            let mut obligations = Obligations::new(proof.regions.start);
            obligations.regions.end = proof.regions.end;
            Ok(Selection::Proven(Evidence {
                provisional_size: false,
                source,
                proof: obligations,
                sized: Vec::new(),
            }))
        }
        Assumed::Deferred => Ok(Selection::Deferred),
        Assumed::Absent => Ok(Selection::Absent),
    }
}

fn inference(error: infer::Error) -> String {
    error.to_string()
}
