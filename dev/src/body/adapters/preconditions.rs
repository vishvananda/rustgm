//! Declared adapter premises come only from the checked source execution interface.
#[cfg(test)]
#[path = "../../../tests/invariants/wrapper_premises.rs"]
mod tests;
use crate::{
    body::{invocation_entries, regions::Contract},
    sema::{
        closures::{Body, Execution},
        model::Declarations,
    },
    types::{RegionId, Store, closure::Adapter, outlives::Pair},
};

pub(super) struct Requirements(Vec<Pair>);
impl Requirements {
    pub(super) fn pairs(&self) -> &[Pair] {
        &self.0
    }
    pub(super) fn validate(&self, input: &Contract) -> Result<(), &'static str> {
        if input.assumptions != self.0
            || !input.type_assumptions.is_empty()
            || !input.parameters.is_empty()
            || input.output.is_some()
            || input.builtins.is_some()
        {
            return Err("receiver adapter has undeclared interface assumptions");
        }
        Ok(())
    }
}
pub(super) fn selected(
    facts: &Declarations,
    source: Body,
    canonical: Execution,
    adapter: Adapter,
) -> Result<Requirements, &'static str> {
    if !source
        .owner
        .is_some_and(|owner| facts.generics.has_parameters(owner))
    {
        return Ok(Requirements(Vec::new()));
    }
    let pairs = facts
        .closures
        .conditions(source.identity)
        .ok_or("adapter requires checked source execution preconditions")?;
    validate(&facts.types, canonical, source.universals, adapter, pairs)?;
    Ok(Requirements(pairs.to_vec()))
}
fn validate(
    types: &Store,
    canonical: Execution,
    original: u32,
    adapter: Adapter,
    pairs: &[Pair],
) -> Result<(), &'static str> {
    // Only free body slots occurring in complete Self can be interface inputs.
    // The matcher leaves callable-local binders rigid, so public input slots
    // cannot become premises through their nested signature occurrence.
    let bindings =
        invocation_entries::source_bindings(types, canonical, adapter.instance(), original)?;
    for &(a, b) in pairs {
        for region in [a, b] {
            if region == RegionId::STATIC {
                continue;
            }
            let Some((0, slot)) = region.bound_parts() else {
                return Err("adapter precondition is not an external source lifetime");
            };
            if u32::from(slot) >= original
                || bindings
                    .slots
                    .binary_search_by_key(&slot, |&(slot, _)| slot)
                    .is_err()
            {
                return Err("adapter precondition does not occur in external Self");
            }
        }
    }
    Ok(())
}
