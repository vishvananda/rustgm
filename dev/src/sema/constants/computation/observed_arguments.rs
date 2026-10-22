//! Complete bounded argument observations are shared; source-specific proofs are not.
use super::{CallerInstance, Declarations, namespaces::Namespaces};
use crate::{
    sema::inline_constants::Context,
    types::{RegionId, substitute::Bound},
};

pub(super) struct Observed {
    regions: Vec<RegionId>,
}
fn context(facts: &Declarations, enclosing: CallerInstance) -> Result<Context, &'static str> {
    let context = match enclosing {
        CallerInstance::Callable(instance) => Ok(Context::Callable(instance)),
        CallerInstance::Initializer(instance) => Ok(Context::Initializer(instance)),
        CallerInstance::Inline(instance) => Ok(instance.enclosing()),
        CallerInstance::Original => Err("source namespace requires an exact instance"),
    }?;
    Ok(match context {
        Context::SourceInitializer(witness) => Context::Initializer(witness.identity(facts)?.0),
        context => context,
    })
}
pub(super) fn regions(
    facts: &Declarations,
    enclosing: CallerInstance,
) -> Result<&[RegionId], &'static str> {
    let key = context(facts, enclosing)?;
    facts
        .source_computations
        .as_deref()
        .and_then(|c| c.0.namespaces.as_deref())
        .and_then(|n| n.observations.get(&key))
        .map(|row| row.regions.as_slice())
        .ok_or("source namespace has no complete argument observation")
}
pub(super) fn prepare(
    facts: &mut Declarations,
    enclosing: CallerInstance,
    scratch: usize,
) -> Result<(), &'static str> {
    let key = context(facts, enclosing)?;
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
    if regions(facts, enclosing).is_ok() {
        return Ok(());
    }
    let catalog = &facts
        .source_computations
        .as_deref()
        .ok_or("missing source observation catalog")?
        .0;
    if catalog
        .namespaces
        .as_ref()
        .is_some_and(|n| n.observations.len() >= 1_000_000)
    {
        return Err("source observation row count budget exceeded");
    }
    let metadata = crate::support::retained::new_map::<Context, Observed>(1).saturating_add(
        if catalog.namespaces.is_none() {
            size_of::<Namespaces>()
        } else {
            0
        },
    );
    let bytes = catalog
        .namespace_available()
        .saturating_sub(metadata)
        .saturating_sub(scratch);
    let mut fold = Bound::observe_budgeted_regions(1_000_000, bytes)?;
    let result = (|| {
        match key {
            Context::Callable(instance) => {
                fold.apply(&mut facts.types, instance.ty())?;
            }
            Context::SourceInitializer(_) => {
                return Err("source observation key was not canonicalized");
            }
            Context::Initializer(instance) => {
                fold.apply_arguments(&mut facts.types, instance.arguments)?;
            }
        }
        fold.collect_observed_free_regions()
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(fold.steps);
    // A failed or partially observed graph is never a reusable row.
    let regions = result?;
    drop(fold);
    let payload = crate::support::retained::vector(&regions);
    let catalog = &mut facts
        .source_computations
        .as_deref_mut()
        .ok_or("missing source observation catalog")?
        .0;
    let planned = catalog
        .namespaces
        .as_ref()
        .map_or(0, |n| n.bytes)
        .saturating_add(metadata)
        .saturating_add(payload);
    catalog.namespace_budget(planned.saturating_add(scratch))?;
    let namespaces = catalog
        .namespaces
        .get_or_insert_with(|| Box::new(Namespaces::new()));
    if namespaces
        .observations
        .insert(key, Observed { regions })
        .is_some()
    {
        return Err("source observation replaced an immutable row");
    }
    namespaces.bytes = planned;
    facts.constant_work.source_context_bytes = catalog.retained_bytes();
    Ok(())
}

#[cfg(test)]
#[path = "../../../../tests/invariants/observed_arguments.rs"]
mod tests;
