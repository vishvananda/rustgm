//! Semantic const eligibility joins the same initialized plan before normal loans.
use crate::{
    body::{
        self,
        ir::Body,
        regions::{Contract, Contracts},
        templates::Templates,
    },
    borrow::{self, loans::templates::Verifier, regions},
    sema::model::Declarations,
    types::{ItemId, copy::Copies},
};
type Result<T> = std::result::Result<T, String>;

pub(super) fn one(
    facts: &mut Declarations,
    body: &Body,
    destruction: &body::drops::Requirements,
    input: &mut super::execution::LoanInput<'_, '_>,
    verifier: &mut Verifier,
) -> Result<()> {
    let super::execution::LoanInput {
        parameters,
        contract,
        regions,
        purpose,
    } = input;
    let required = body.const_context || contract.requires_defining_cleanup_preparation();
    let mut copies = Copies::default();
    if facts.source_bindings.is_none()
        || !required
        || borrow::loans::templates::no_cleanup(destruction)
    {
        return with_environment(facts, &mut copies, |environment| {
            verifier.body(
                environment,
                body,
                destruction,
                parameters,
                regions,
                &|ty, field| body::drops::nominal_position(facts, ty, field),
            )
        })
        .map(|_| ())
        .map_err(|e| e.message);
    }
    facts.types.arena_identity();
    facts.aggregates.arena_identity();
    let plan = with_environment(facts, &mut copies, |environment| {
        verifier.prepare(
            environment,
            body,
            destruction,
            parameters,
            regions,
            &|ty, field| body::drops::nominal_position(facts, ty, field),
        )
    })
    .map_err(|e| e.message)?;
    let active = plan.active().next().is_some();
    let eligible = plan.active_const()?;
    if eligible {
        body::defining_cleanups::select(facts, body, contract.to_mut(), plan.eligibility())?;
        **regions = recheck(facts, body, contract)?;
    }
    with_environment(facts, &mut copies, |environment| {
        if eligible {
            verifier.finish(environment, plan, regions)
        } else {
            verifier.finish_initial(environment, plan, regions)
        }
    })
    .map_err(|e| e.message)?;
    if active && *purpose == super::execution::ProofPurpose::Code {
        return Err("constant interpreter cleanup scheduling is not implemented".into());
    }
    Ok(())
}
pub(crate) fn templates<'a>(
    facts: &mut Declarations,
    inputs: Vec<Contract>,
    templates: &'a Templates,
    contexts: &'a borrow::templates::Contexts,
    custom: &[ItemId],
) -> Result<(borrow::templates::Checked<'a>, Vec<Contract>)> {
    let required = facts.source_bindings.is_some()
        && templates
            .iter()
            .chain(&contexts.bodies.bodies)
            .any(|template| {
                template.body.const_context
                    && !borrow::loans::templates::no_cleanup(&template.destruction)
            });
    let mut copies = Copies::default();
    if !required {
        let contracts = Contracts {
            bodies: inputs,
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        };
        let checked = with_environment(facts, &mut copies, |environment| {
            borrow::templates::verify_with_context(
                environment,
                &facts.lifetimes.variances,
                &contracts,
                templates,
                contexts,
                custom,
                |ty, field| body::drops::nominal_position(facts, ty, field),
            )
        })
        .map_err(|e| e.message)?;
        return Ok((checked, contracts.bodies));
    }
    facts.types.arena_identity();
    facts.aggregates.arena_identity();
    let (mut batch, inputs) = {
        let contracts = Contracts {
            bodies: inputs,
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        };
        let batch = with_environment(facts, &mut copies, |environment| {
            borrow::templates::prepare_with_context(
                environment,
                &facts.lifetimes.variances,
                &contracts,
                templates,
                contexts,
                custom,
            )
        })
        .map_err(|e| e.message)?;
        (batch, contracts.bodies)
    };
    loop {
        let row = with_environment(facts, &mut copies, |environment| {
            batch.next(environment, &|ty, field| {
                body::drops::nominal_position(facts, ty, field)
            })
        })
        .map_err(|e| e.message)?;
        let Some(mut row) = row else {
            break;
        };
        if row.active()? {
            let (plan, contract) = row.input();
            body::defining_cleanups::select(facts, plan.body(), contract, plan.eligibility())?;
            contract
                .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
            let mut queries = crate::types::outlives::Queries::new(
                &facts.types,
                &facts.lifetimes.nominal_requirements,
                &facts.lifetimes.callables,
            );
            row.refresh(&facts.types, &facts.lifetimes.variances, &mut queries)
                .map_err(|e| e.message)?;
        }
        with_environment(facts, &mut copies, |environment| row.finish(environment))
            .map_err(|e| e.message)?;
    }
    batch
        .finish(&facts.types)
        .map(|checked| (checked, inputs))
        .map_err(|e| e.message)
}
pub(super) fn recheck(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
) -> Result<regions::Body> {
    contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        contract,
        body,
    )
    .map_err(|e| e.message)
}
fn with_environment<T>(
    facts: &Declarations,
    copies: &mut Copies,
    work: impl FnOnce(&mut borrow::Environment<'_, crate::support::hash::BuildFxHasher>) -> T,
) -> T {
    work(&mut borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies,
    })
}

/// The native owner and its recipe cache retain the real initialized concrete plan.
pub(crate) fn ordinary(
    facts: &mut Declarations,
    body: Body,
    input: &Contract,
    mut regions: regions::Body,
    verifier: &mut borrow::incremental::Verifier,
    cache: &mut Option<borrow::destruction::glue::Cache>,
) -> std::result::Result<borrow::incremental::Proved, borrow::Error> {
    borrow::incremental::with_const_original(body, |body| {
        let error = |message: &str| borrow::Error {
            node: Some(body.source.into()),
            message: message.into(),
        };
        facts.types.arena_identity();
        facts.aggregates.arena_identity();
        let mut copies = Copies::default();
        let recipes = cache
            .take()
            .ok_or_else(|| error("const original lost its actual recipe cache"))?;
        let mut catalog = recipes.attach(&facts.types, &facts.aggregates, &facts.members);
        let plan = with_environment(facts, &mut copies, |environment| {
            verifier.prepare_const_original(environment, body, &regions, &mut catalog)
        });
        *cache = Some(catalog.into_cache());
        let plan = plan?;
        let mut selected = None;
        if plan.eligibility().next().is_some() {
            let mut contract = input.clone();
            body::defining_cleanups::select(facts, body, &mut contract, plan.eligibility())
                .map_err(|e| error(&e))?;
            regions = recheck(facts, body, &contract).map_err(|e| error(&e))?;
            selected = Some(contract);
        }
        // Keep the selected contract alive through exact Copy/state authentication.
        let _input = selected.as_ref().unwrap_or(input);
        with_environment(facts, &mut copies, |environment| {
            verifier.finish_const_original(environment, plan, &regions, selected.is_some())
        })
    })
}
