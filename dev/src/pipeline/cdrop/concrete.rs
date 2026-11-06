//! A concrete Code plan pays its actual recipe inventory and the ordinary loan join.
use super::{Result, body, borrow, with_environment};
use crate::{
    borrow::loans::templates::{Initial, Verifier},
    resolve::expansion::Report,
    sema::model::Declarations,
    types::copy::Copies,
};

pub(super) fn one<'body>(
    report: &Report,
    facts: &mut Declarations,
    initial: Initial<'body>,
    input: &mut crate::pipeline::execution::LoanInput<'_, '_>,
    verifier: &mut Verifier,
    copies: &mut Copies,
) -> Result<super::Checked<'body>> {
    let body = initial.body();
    let count = facts.destructors.implementations().len();
    let bytes = count
        .checked_mul(size_of::<body::drops::Method>())
        .ok_or("constant cleanup method inventory storage overflow")?;
    facts.charge_caller_inputs(count, bytes)?;
    let mut methods = Vec::with_capacity(count);
    for implementation in facts.destructors.implementations() {
        methods.push(body::drops::method(facts, implementation)?);
    }
    let mut catalog = borrow::destruction::glue::Catalog::new(
        &facts.types,
        &facts.aggregates,
        &facts.members,
        &methods,
    )?;
    let selected = with_environment(facts, copies, |environment| {
        verifier.select_execution(environment, initial, input.regions, &mut catalog)
    });
    let recipe_steps = catalog.steps();
    let cache = catalog.into_cache();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(recipe_steps);
    let mut plan = selected.map_err(|error| error.message)?;
    let eligible = super::eligibility(report, facts, &mut plan, input.contract, input.regions)?;
    let execution = with_environment(facts, copies, |environment| {
        verifier.finish_execution(environment, plan, input.regions, eligible)
    })
    .map_err(|error| error.message)?;
    with_environment(facts, copies, |environment| {
        verifier.require_execution(environment, &execution, input.regions)
    })?;
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(execution.planning_steps());
    if execution.active() {
        facts.charge_caller_inputs(
            1,
            size_of::<(
                crate::borrow::loans::templates::Execution<'_>,
                borrow::destruction::glue::Cache,
            )>(),
        )?;
        let mut checked = super::Checked::new(body, input.purpose, input.regions);
        checked.cleanup = Some(Box::new((execution, cache)));
        return Ok(checked);
    }
    drop(cache);
    Ok(super::Checked::new(body, input.purpose, input.regions))
}
