//! Complete bounded argument observations are shared; source-specific proofs are not.
use super::{CallerInstance, Declarations, namespaces::Namespaces};
use crate::{
    sema::inline_constants::Context,
    types::{RegionId, substitute::Bound},
};

/// Derived canonical argument owner only; no body, namespace or proof permission.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum ArgumentOwner {
    Callable(crate::types::instance::Instance),
    Initializer(crate::sema::values::Instance),
    AsyncBody(crate::types::TypeId),
}
const _: [(); 12] = [(); size_of::<Context>()];
const _: [(); size_of::<Context>()] = [(); size_of::<ArgumentOwner>()];
const _: [(); 20] = [(); size_of::<CallerInstance>()];

pub(super) struct Observed {
    regions: Vec<RegionId>,
}
const _: [(); 40] = [(); size_of::<(Context, Observed)>()];
const _: [(); size_of::<(Context, Observed)>()] = [(); size_of::<(ArgumentOwner, Observed)>()];
fn context(facts: &Declarations, enclosing: CallerInstance) -> Result<ArgumentOwner, &'static str> {
    Ok(match enclosing {
        CallerInstance::AsyncBody(future) => {
            facts.checked_future_definition(future)?;
            ArgumentOwner::AsyncBody(future)
        }
        CallerInstance::Callable(instance) => ArgumentOwner::Callable(instance),
        CallerInstance::Initializer(instance) => ArgumentOwner::Initializer(instance),
        CallerInstance::Inline(instance) => match instance.enclosing() {
            Context::Callable(instance) => ArgumentOwner::Callable(instance),
            Context::Initializer(instance) => ArgumentOwner::Initializer(instance),
            Context::SourceInitializer(witness) => {
                ArgumentOwner::Initializer(witness.identity(facts)?.0)
            }
        },
        CallerInstance::Original => return Err("source namespace requires an exact instance"),
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
    let metadata = crate::support::retained::new_map::<ArgumentOwner, Observed>(1).saturating_add(
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
            ArgumentOwner::AsyncBody(future) => {
                let Some(crate::types::Kind::Future { arguments, .. }) = facts.types.kind(future)
                else {
                    return Err("async observation lost its genuine future application");
                };
                fold.apply_arguments(&mut facts.types, arguments)?;
            }
            ArgumentOwner::Callable(instance) => {
                fold.apply(&mut facts.types, instance.ty())?;
            }
            ArgumentOwner::Initializer(instance) => {
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
