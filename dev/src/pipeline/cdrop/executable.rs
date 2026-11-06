//! The pipeline consumes completed borrow evidence before projecting immutable data.
use crate::{
    body::{
        drops::{glue, schedule::View},
        ir::Body,
        regions::Contract,
    },
    borrow::{self, destruction::timeline, loans::templates::Execution},
    sema::model::Declarations,
};
pub(in crate::pipeline) fn view(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    regions: &borrow::regions::Body,
    (execution, cache): (Execution<'_>, glue::Cache),
    budget: usize,
    work: &mut usize,
) -> Result<View, &'static str> {
    let mut copies = crate::types::copy::Copies::default();
    execution.require_current(
        &borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            source_copy: facts.uses_source_copy(),
            copies: &mut copies,
        },
        body,
        regions,
    )?;
    let state = contract
        .builtins
        .as_deref()
        .and_then(|b| b.defining())
        .ok_or("executable cleanup requires its actual defining State")?;
    let mut proof_work = crate::body::defining_cleanups::Work::new(1_000_000);
    let checked = state.checked(
        body,
        contract
            .builtins
            .as_deref()
            .and_then(|b| b.environment.as_ref()),
        &mut proof_work,
    );
    *work = work.saturating_add(proof_work.steps);
    checked?;
    proof_work.steps = 0;
    let (_, plan) = execution.into_plan();
    let plan = plan.ok_or("executable cleanup lost its actual concrete plan")?;
    let recipes = cache.into_recipes();
    let charge = crate::body::copy_requirements::OriginalIndexBudget::new(1_000_000, budget);
    let result = (|| {
        charge.charge(body.blocks.len(), 0)?;
        let index_bytes = timeline::Index::allocation_bound(body, &plan.events)?;
        charge.charge(plan.flags.len(), 0)?;
        if plan.flags.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err("execution flags changed their actual sorted plan domain");
        }
        let flag_count =
            u32::try_from(plan.flags.len()).map_err(|_| "execution flag extent overflow")?;
        let words = plan.cleanups.len().div_ceil(64);
        let fixed = size_of::<View>()
            .saturating_add(plan.allocation_bytes())
            .saturating_add(recipes.allocation_bytes())
            .saturating_add(words.saturating_mul(size_of::<u64>()));
        charge.charge(
            plan.events.len().saturating_add(plan.cleanups.len()),
            fixed.saturating_add(index_bytes),
        )?;
        let calls = super::profile::select(
            facts,
            body,
            contract,
            &plan,
            &recipes,
            &charge,
            &mut proof_work,
        )?;
        let timeline = timeline::Index::new(body, &plan.events)?;
        if timeline.allocation_bytes() > index_bytes {
            return Err("execution timeline exceeded its prepaid storage bound");
        }
        let mut active = vec![0u64; words];
        for (index, cleanup) in plan.cleanups.iter().enumerate() {
            if cleanup.node.is_some() {
                active[index / 64] |= 1 << (index % 64);
            }
        }
        let cleanup_count =
            u32::try_from(plan.cleanups.len()).map_err(|_| "execution cleanup extent overflow")?;
        let (blocks, edges) = timeline.into_lanes();
        let view = View {
            events: plan.events,
            recipes,
            calls,
            active,
            cleanup_count,
            flag_count,
            blocks,
            edges,
        };
        charge.charge(
            view.calls
                .len()
                .saturating_add(view.active.len())
                .saturating_add(view.events.len()),
            0,
        )?;
        view.validate()?;
        Ok(view)
    })();
    *work = work
        .saturating_add(proof_work.steps)
        .saturating_add(charge.costs().0);
    result
}
