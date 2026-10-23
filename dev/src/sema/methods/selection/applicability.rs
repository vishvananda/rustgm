//! Bind each candidate once; applicability never authorizes a completed body.
use super::{Caller, Receiver};
use crate::{
    resolve::model::DefId,
    sema::{
        model::Declarations,
        traits::{
            assumptions::Assumptions,
            selection::{self, Selection},
        },
    },
    types::{
        infer::{self, Context, Instantiation, TermId, arguments::Argument, properties::Query},
        parameters::{ParameterKind, environment::Environment, predicates::Bound},
    },
};
use std::sync::Arc;

pub(super) struct Conditions {
    pub builtins: Vec<(TermId, Bound)>,
    pub traits: Vec<infer::predicates::Predicate>,
}

pub(in crate::sema::methods) fn applicable(
    facts: &mut Declarations,
    context: &mut Context,
    prefix: TermId,
    owner: DefId,
    receiver: Option<Receiver>,
    caller: Caller<'_>,
) -> Result<bool, &'static str> {
    let Some(conditions) = prepare(facts, context, prefix, owner, receiver)? else {
        return Ok(false);
    };
    if !trait_requirements(facts, context, &conditions.traits, caller)? {
        return Ok(false);
    }
    builtin_requirements(facts, context, &conditions.builtins, caller.environment)
}

pub(super) fn prepare(
    facts: &mut Declarations,
    context: &mut Context,
    prefix: TermId,
    owner: DefId,
    receiver: Option<Receiver>,
) -> Result<Option<Conditions>, &'static str> {
    let target = facts
        .associated
        .schema(owner)
        .ok_or("missing candidate schema")?
        .self_type;
    let subject = target;
    let target = receiver.as_ref().map_or(target, |r| r.ty);
    let ids = if facts.associated.trait_owner(owner).is_some()
        || facts.associated.operation(owner).is_some()
        || facts.associated.comparison(owner).is_some()
        || facts
            .iteration
            .as_deref()
            .is_some_and(|r| r.owns_trait(owner))
        || facts
            .cloning
            .as_deref()
            .is_some_and(|r| r.owns_trait(owner))
    {
        facts.generics.callable_parameters(&facts.types, owner)?.1
    } else {
        facts.generics.parameter_ids(&facts.types, owner)?
    };
    let mut bindings = Vec::with_capacity(ids.len());
    let mut obligations = Vec::new();
    let mut traits = Vec::new();
    for id in ids {
        let argument = context
            .parameter_variable(&facts.types, id)
            .map_err(inference)?;
        if let Argument::Type(term) = argument
            && matches!(
                facts.types.parameter(id).map(|p| p.kind),
                Some(ParameterKind::Type { sized: true })
            )
        {
            obligations.push((term, Bound::Sized));
        }
        bindings.push((id, argument));
    }
    let mut substitution =
        Instantiation::new(context, &facts.types, &bindings).map_err(inference)?;
    let target = context
        .instantiate_with(&mut facts.types, target, &mut substitution)
        .map_err(inference)?;
    if let Some(actual) = receiver.and_then(|r| r.subject) {
        let subject = context
            .instantiate_with(&mut facts.types, subject, &mut substitution)
            .map_err(inference)?;
        match context.equate_shape(&facts.types, subject, actual) {
            Ok(()) => {}
            Err(infer::Error::Mismatch | infer::Error::Occurs) => return Ok(None),
            Err(error) => return Err(inference(error)),
        }
    }
    match context.equate_shape(&facts.types, target, prefix) {
        Ok(()) => {}
        Err(infer::Error::Mismatch | infer::Error::Occurs) => return Ok(None),
        Err(e) => return Err(inference(e)),
    }
    for predicate in facts.generics.predicates_in_scope(owner) {
        let predicate = predicate.value.ordinary()?;
        // Applicability checks the abstract marker and Sized requirements. The
        // chosen callable retains and proves its complete canonical trait goal.
        if facts
            .cloning
            .as_deref()
            .is_some_and(|r| r.owns_trait(owner))
            && facts.generics.trait_self_bounds.get(&owner) == Some(&predicate)
        {
            continue;
        }
        if matches!(
            predicate.bound,
            Bound::Trait(_)
                | Bound::ConstTrait(_)
                | Bound::MaybeConstTrait(_)
                | Bound::ProjectionEqual(_)
                | Bound::Auto(_)
        ) {
            traits.push(
                context
                    .instantiate_predicate(&mut facts.types, predicate, &mut substitution)
                    .map_err(inference)?,
            );
            continue;
        }
        if matches!(predicate.bound, Bound::Outlives(_)) {
            continue;
        }
        let term = context
            .instantiate_with(&mut facts.types, predicate.ty, &mut substitution)
            .map_err(inference)?;
        obligations.push((term, predicate.bound));
    }
    Ok(Some(Conditions {
        builtins: obligations,
        traits,
    }))
}

pub(super) fn builtin_requirements(
    facts: &mut Declarations,
    context: &mut Context,
    obligations: &[(TermId, Bound)],
    environment: Option<&Arc<Environment>>,
) -> Result<bool, &'static str> {
    if obligations.is_empty() {
        return Ok(true);
    }
    let environment = if let Some(environment) = environment {
        Arc::clone(environment)
    } else {
        Arc::new(Environment::new(&facts.types, &[])?)
    };
    let mut properties = Query::new(environment);
    for &(term, bound) in obligations {
        let value = properties.demand(
            context,
            &mut facts.types,
            &facts.aggregates,
            &facts.derives,
            term,
        )?;
        let possible = match bound {
            Bound::Sized => value.sized,
            Bound::Copy => value.abilities.copy,
            Bound::Clone => value.abilities.clone_,
            Bound::Outlives(_) => return Err("unexpected candidate region predicate"),
            Bound::Trait(_) | Bound::ConstTrait(_) | Bound::MaybeConstTrait(_) | Bound::Auto(_) => {
                return Err("user trait candidate obligations require trait selection");
            }
            Bound::ProjectionEqual(_) => {
                return Err("projection equality requires a normalization proof");
            }
        };
        if !possible {
            return Ok(false);
        }
    }
    Ok(true)
}
fn trait_requirements(
    facts: &mut Declarations,
    context: &mut Context,
    goals: &[infer::predicates::Predicate],
    caller: Caller<'_>,
) -> Result<bool, &'static str> {
    if goals.is_empty() {
        return Ok(true);
    }
    let empty = Assumptions::default();
    let assumptions = caller.traits.unwrap_or(&empty);
    let mut first_region = facts
        .body_regions
        .probe_frontier()
        .map_err(|_| "method trait region identity space exhausted")?;
    let mut pending: Vec<_> = goals.iter().collect();
    while !pending.is_empty() {
        let before = pending.len();
        let mut next = Vec::new();
        for goal in pending {
            match selection::select_available(
                facts,
                context,
                goal,
                selection::Caller {
                    assumptions,
                    environment: caller.environment,
                    first_region,

                    retain_sized: false,
                },
                caller.availability,
            )
            .map_err(|_| "invalid or exhausted method trait selection")?
            {
                Selection::Absent => return Ok(false),
                Selection::Deferred => next.push(goal),
                Selection::Proven(proof) => first_region = proof.proof.regions.end,
            }
        }
        if next.len() == before {
            // A candidate is possible; the selected ordinary callable must prove
            // its bounds after all arguments and result constraints are known.
            return Ok(true);
        }
        pending = next;
    }
    Ok(true)
}
pub(in crate::sema::methods) fn inference(error: infer::Error) -> &'static str {
    match error {
        infer::Error::Budget => "method candidate work budget exceeded",
        infer::Error::Overflow => "method candidate arena exceeds 32 bits",
        _ => "invalid method candidate inference graph",
    }
}
