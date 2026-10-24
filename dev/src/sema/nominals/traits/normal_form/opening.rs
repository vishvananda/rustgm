//! Open a projection's caller aliases exactly once, retaining their restoration equations.
use super::{Availability, Declarations, Environment, Obligations, Predicate, State};
use crate::types::{GenericArgs, substitute::Bound};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../../../tests/invariants/closed_callers.rs"]
mod tests;

pub(super) fn prepare(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    inputs: (Predicate, GenericArgs),
    first: u32,
    retain_sized: bool,
    availability: Availability,
) -> Result<(State, Predicate, GenericArgs), &'static str> {
    let (requirement, arguments) = inputs;
    if environment
        .as_ref()
        .is_none_or(|env| !env.requires_region_mapping())
        && !requirement.has_free_regions(&facts.types)?
        && !facts.types.arguments_have_free_regions(arguments)?
    {
        let mut state = State::new(facts, environment, None)?;
        state.obligations = Obligations::new(first);
        state.retain_sized = retain_sized;
        state.availability = availability;
        return Ok((state, requirement, arguments));
    }
    let mut fold = Bound::open_regions(first)?;
    let requirement = fold.predicate(&mut facts.types, requirement)?;
    let arguments = fold.apply_arguments(&mut facts.types, arguments)?;
    let environment = environment
        .map(|env| {
            let shared = if env.requires_region_mapping() {
                let before = fold.steps;
                let outcome = env.identity_region_opening(&mut facts.types, &mut fold);
                facts.body_work.canonical_steps = facts
                    .body_work
                    .canonical_steps
                    .saturating_add(fold.steps.saturating_sub(before));
                outcome?
            } else {
                true
            };
            if shared {
                // Opening the request does not change this caller environment.
                Ok(env)
            } else {
                let before = fold.steps;
                let result = env.mapped_trait_opening(&mut facts.types, &mut fold);
                facts.body_work.canonical_steps = facts
                    .body_work
                    .canonical_steps
                    .saturating_add(fold.steps.saturating_sub(before));
                if let Some(mapped) = result? {
                    return Ok(mapped);
                }
                let before = fold.steps;
                let result = env.substitute_regions(&mut facts.types, &mut fold);
                facts.body_work.canonical_steps = facts
                    .body_work
                    .canonical_steps
                    .saturating_add(fold.steps.saturating_sub(before));
                result.map(Arc::new)
            }
        })
        .transpose()?;
    let mut work = 0;
    let result = State::new_counted(facts, environment, &mut work);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    let mut state = result?;
    state.retain_sized = retain_sized;
    state.availability = availability;
    state
        .context
        .account_work(fold.steps)
        .map_err(|_| "projection proof work exhausted")?;
    state.obligations.regions = first
        ..fold
            .next_region()
            .ok_or("missing projection region interval")?;
    let mut opened: Vec<_> = fold.opened_regions().map(|(a, b)| (b, a)).collect();
    opened.sort_unstable_by_key(|(free, original)| (free.0, original.0));
    state.obligations.equalities.extend(opened);
    Ok((state, requirement, arguments))
}
