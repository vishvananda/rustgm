//! Certify that a partial query cannot infer inputs before they become canonical.
use super::{Caller, Context, Declarations, inference, instantiation};
use crate::{
    sema::traits::{heads::Subject, implementations::Availability},
    types::infer::{Error, TermId, probe::Probe},
};

/// Unknown Self cannot select a caller assumption or an implementation header.
/// Wait for its head or numeric domain before opening a speculative query.
pub(in crate::sema) fn has_known_subject(
    store: &crate::types::Store,
    context: &mut Context,
    subject: TermId,
) -> Result<bool, Error> {
    match Subject::query(store, context, subject) {
        Ok(subject) => Ok(subject.is_some()),
        Err(Error::Normalization) => Ok(false),
        Err(error) => Err(error),
    }
}

/// A single input-preserving header with only builtin premises cannot complete
/// unresolved input equations. This is scheduling information, never trait proof.
/// Other headers, caller assumptions and recursive trait premises keep retrying.
pub(in crate::sema) fn requires_resolved_inputs(
    facts: &mut Declarations,
    context: &mut Context,
    root: TermId,
    caller: &Caller<'_>,
) -> Result<bool, String> {
    let application = context
        .projection_application(&facts.types, root)
        .map_err(inference)?;
    if !application.arguments.is_empty() {
        return Ok(false);
    }
    let goal = &application.predicate;
    let (item, _) = goal
        .trait_arguments()
        .ok_or("missing projected trait goal")?;
    let Some(subject) = Subject::query(&facts.types, context, goal.subject()).map_err(inference)?
    else {
        return Ok(false);
    };
    if caller.assumptions.may_have_subject(item, subject) {
        return Ok(false);
    }
    let Some(registry) = &mut facts.generics.implementations else {
        return Ok(false);
    };
    let candidates = registry
        .candidates(&mut facts.types, context, goal, Availability::Validated)
        .map_err(inference)?;
    let [owner] = candidates.as_slice() else {
        return Ok(false);
    };
    if transparent_subject(facts, context, goal, *owner).map_err(inference)? {
        return Ok(true);
    }
    let (mut probe, roots) = Probe::new(&facts.types, context, &[root]).map_err(inference)?;
    let input = roots[0];
    let trial = probe
        .context()
        .projection_application(&facts.types, input)
        .map_err(inference)?;
    let instance = instantiation::instantiate(facts, probe.context(), *owner, caller.first_region)?;
    probe
        .context()
        .account_work(instance.goals.len())
        .map_err(inference)?;
    let result = if instance
        .goals
        .iter()
        .any(|goal| goal.builtin_bound().is_none())
    {
        false
    } else {
        match probe
            .context()
            .equate_predicates(&facts.types, &trial.predicate, &instance.goal)
        {
            Ok(()) => probe.preserves_inputs(context).map_err(inference)?,
            Err(Error::Mismatch | Error::Occurs | Error::Normalization) => false,
            Err(error) => return Err(inference(error)),
        }
    };
    context
        .account_work(probe.context().steps)
        .map_err(inference)?;
    Ok(result)
}

/// An independent type parameter beneath already-known reference/pointer heads
/// cannot refine any input. Other layouts retain the general isolated probe.
fn transparent_subject(
    facts: &Declarations,
    context: &mut Context,
    goal: &crate::types::infer::predicates::Predicate,
    owner: crate::resolve::model::DefId,
) -> Result<bool, Error> {
    use crate::types::parameters::predicates::Bound;
    for predicate in facts.generics.declared_predicates_in_scope(owner) {
        context.account_work(1)?;
        if matches!(
            predicate.value.bound,
            Bound::Trait(_) | Bound::ProjectionEqual(_)
        ) {
            return Ok(false);
        }
    }
    super::header_inputs::independent(facts, context, goal, owner).map(|inputs| inputs.is_some())
}
