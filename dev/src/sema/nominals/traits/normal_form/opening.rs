//! Open a projection's caller aliases exactly once, retaining their restoration equations.
use super::{Availability, Declarations, Environment, Obligations, Predicate, State};
use crate::types::substitute::Bound;
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../../../tests/invariants/closed_callers.rs"]
mod tests;

pub(super) fn prepare(
    facts: &mut Declarations,
    environment: Option<Arc<Environment>>,
    requirement: Predicate,
    first: u32,
    retain_sized: bool,
    availability: Availability,
) -> Result<(State, Predicate), &'static str> {
    if environment
        .as_ref()
        .is_none_or(|env| !env.requires_region_mapping())
        && !requirement.has_free_regions(&facts.types)?
    {
        let mut state = State::new(facts, environment, None)?;
        state.obligations = Obligations::new(first);
        state.retain_sized = retain_sized;
        state.availability = availability;
        return Ok((state, requirement));
    }
    let mut fold = Bound::open_regions(first)?;
    let requirement = fold.predicate(&mut facts.types, requirement)?;
    let environment = environment
        .map(|env| {
            if env.requires_region_mapping() {
                env.substitute_regions(&mut facts.types, &mut fold)
                    .map(Arc::new)
            } else {
                // Opening the request does not change a closed caller environment.
                Ok(env)
            }
        })
        .transpose()?;
    let mut state = State::new(facts, environment, None)?;
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
    Ok((state, requirement))
}
