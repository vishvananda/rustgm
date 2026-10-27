//! One actual executable search survives selected declaration dependencies.
use super::{self as selection, Declarations};
use crate::types::infer::Context;

pub(in crate::sema) fn select_prepared(
    facts: &mut Declarations,
    context: &mut Context,
    goal: &crate::types::infer::predicates::Predicate,
    caller: &selection::Caller<'_>,
    preparation: &mut dyn crate::sema::dispatch::Preparation,
) -> Result<selection::Selection, crate::sema::dispatch::Failure> {
    use selection::{OwnedCaller, Progress};
    preparation
        .prepay(
            facts,
            1,
            size_of::<crate::sema::traits::assumptions::Assumptions>()
                .saturating_add(2 * size_of::<usize>()),
        )
        .map_err(crate::sema::dispatch::Failure::Preparation)?;
    let caller = OwnedCaller {
        assumptions: std::sync::Arc::new(caller.assumptions.clone()),
        environment: caller.environment.cloned(),
        first_region: caller.first_region,
        retain_sized: caller.retain_sized,
    };
    let mut work = 0;
    let progress = selection::begin_executable_counted(facts, context, goal, caller, &mut work);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    preparation
        .prepay(facts, work, 0)
        .map_err(crate::sema::dispatch::Failure::Preparation)?;
    let mut progress = progress?;
    loop {
        let (dependency, query) = match progress {
            Progress::Ready(result) => return Ok(result),
            Progress::Inferred { .. } => {
                return Err("concrete dispatch cannot publish partial selection inputs".into());
            }
            Progress::Pending(dependency, query) => (dependency, query),
        };
        let first = query.allocation_end();
        preparation
            .prepay(facts, 1, 0)
            .map_err(crate::sema::dispatch::Failure::Preparation)?;
        preparation
            .prepare(facts, dependency)
            .map_err(crate::sema::dispatch::Failure::Preparation)?;
        if !dependency.ready(facts) {
            return Err("selected dispatch dependency remains pending".into());
        }
        let before = work;
        let next = query.resume_counted(facts, context, goal, first, &mut work);
        let attempted = work.saturating_sub(before);
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(attempted);
        preparation
            .prepay(facts, attempted, 0)
            .map_err(crate::sema::dispatch::Failure::Preparation)?;
        progress = next?;
    }
}
