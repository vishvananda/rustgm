//! Semantic const eligibility joins the same initialized plan before normal loans.
mod concrete;
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
    report: &crate::resolve::expansion::Report,
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
    let mode = if *purpose == super::execution::ProofPurpose::Code {
        borrow::ownership::initialization::Refinement::EligibilityOnly
    } else {
        borrow::ownership::initialization::Refinement::Defining
    };
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
    let mut initial = with_environment(facts, &mut copies, |environment| {
        verifier.begin(environment, body, parameters, regions)
    })
    .map_err(|e| e.message)?;
    if function_qualification_body(facts, body) {
        with_environment(facts, &mut copies, |environment| {
            verifier.begin_qualification(
                environment,
                &mut initial,
                regions,
                (destruction, mode),
                &|ty, field| body::drops::nominal_position(facts, ty, field),
            )
        })?;
        if let Some(questions) = initial.questions(&mut facts.types, &facts.aggregates)? {
            let answers =
                body::defining_cleanups::qualify(facts, body, contract.to_mut(), questions)?;
            **regions = recheck(facts, body, contract)?;
            with_environment(facts, &mut copies, |environment| {
                verifier.accept_qualification(environment, &mut initial, regions, &answers)
            })?;
        }
    }
    if *purpose == super::execution::ProofPurpose::Code {
        return concrete::one(report, facts, initial, input, verifier, &mut copies);
    }
    let mut plan = with_environment(facts, &mut copies, |environment| {
        verifier.select_initial(
            environment,
            initial,
            destruction,
            regions,
            (
                borrow::ownership::initialization::Refinement::Defining,
                &|ty, field| body::drops::nominal_position(facts, ty, field),
            ),
        )
    })
    .map_err(|e| e.message)?;
    let eligible = eligibility(report, facts, &mut plan, contract, regions)?;
    with_environment(facts, &mut copies, |environment| {
        if eligible {
            verifier.finish(environment, plan, regions)
        } else {
            verifier.finish_initial(environment, plan, regions)
        }
    })
    .map_err(|e| e.message)?;
    Ok(())
}
fn eligibility<P: borrow::loans::templates::Payload>(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    plan: &mut borrow::loans::templates::Prepared<'_, P>,
    contract: &mut std::borrow::Cow<'_, Contract>,
    regions: &mut regions::Body,
) -> Result<bool> {
    plan.resolve_selected(&mut facts.types, &facts.aggregates)?;
    let precise = precise_live_drops(report, facts)?;
    let active = plan.active().next().is_some();
    let eligible = plan.active_const()?;
    if eligible {
        body::defining_cleanups::select(
            facts,
            plan.body(),
            contract.to_mut(),
            plan.eligibility(),
            precise,
            active,
        )?;
        *regions = recheck(facts, plan.body(), contract)?;
    }
    Ok(eligible)
}
pub(crate) fn templates<'a>(
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    inputs: Vec<Contract>,
    templates: &'a Templates,
    contexts: &'a borrow::templates::Contexts,
    custom: &[ItemId],
) -> std::result::Result<(borrow::templates::Checked<'a>, Vec<Contract>), borrow::Error> {
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
        })?;
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
        })?;
        (batch, contracts.bodies)
    };
    let precise = precise_live_drops(report, facts).map_err(|message| borrow::Error {
        node: None,
        message,
    })?;
    loop {
        let initial = with_environment(facts, &mut copies, |environment| batch.begin(environment))?;
        let Some(mut initial) = initial else {
            break;
        };
        qualify_row(facts, &mut copies, &mut initial)?;
        let mut row = with_environment(facts, &mut copies, |environment| {
            initial.select(environment, &|ty, field| {
                body::drops::nominal_position(facts, ty, field)
            })
        })?;
        row.resolve_selected(&mut facts.types, &facts.aggregates)?;
        if row.active()? {
            let (plan, contract) = row.input();
            let error = |message| borrow::Error {
                node: Some(plan.body().source.into()),
                message,
            };
            body::defining_cleanups::select(
                facts,
                plan.body(),
                contract,
                plan.eligibility(),
                precise,
                plan.active().next().is_some(),
            )
            .map_err(error)?;
            contract
                .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
                .map_err(|e| error(e.into()))?;
            let mut queries = crate::types::outlives::Queries::new(
                &facts.types,
                &facts.lifetimes.nominal_requirements,
                &facts.lifetimes.callables,
            );
            row.refresh(&facts.types, &facts.lifetimes.variances, &mut queries)?;
        }
        with_environment(facts, &mut copies, |environment| row.finish(environment))?;
    }
    batch.finish(&facts.types).map(|checked| (checked, inputs))
}
fn qualify_row(
    facts: &mut Declarations,
    copies: &mut Copies,
    row: &mut borrow::templates::InitialRow<'_, '_>,
) -> std::result::Result<(), borrow::Error> {
    if !function_qualification_body(facts, row.body()) {
        return Ok(());
    }
    with_environment(facts, copies, |environment| {
        row.begin_qualification(environment, &|ty, field| {
            body::drops::nominal_position(facts, ty, field)
        })
    })?;
    let Some(input) = row.qualification_input(&mut facts.types, &facts.aggregates)? else {
        return Ok(());
    };
    let body = input.body;
    let questions = input.questions;
    let contract = input.contract;
    let error = |message: String| borrow::Error {
        node: Some(body.source.into()),
        message,
    };
    let answers =
        body::defining_cleanups::qualify(facts, body, contract, questions).map_err(error)?;
    contract
        .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)
        .map_err(|message| error(message.into()))?;
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    row.refresh(&facts.types, &facts.lifetimes.variances, &mut queries)?;
    with_environment(facts, copies, |environment| {
        row.accept_qualification(environment, &answers)
    })
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
    report: &crate::resolve::expansion::Report,
    facts: &mut Declarations,
    body: Body,
    (input, requirements): (&Contract, Option<&body::drops::Requirements>),
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
        let mut initial = with_environment(facts, &mut copies, |environment| {
            verifier.begin_const_original(environment, body, &regions)
        })?;
        let mut selected = qualify_native(
            facts,
            input,
            &mut regions,
            verifier,
            (cache, requirements),
            &mut copies,
            &mut initial,
        )?;
        let recipes = cache
            .take()
            .ok_or_else(|| error("const original lost its actual recipe cache"))?;
        let mut catalog = recipes.attach(&facts.types, &facts.aggregates, &facts.members);
        let plan = with_environment(facts, &mut copies, |environment| {
            verifier.select_const_original(environment, initial, &regions, &mut catalog)
        });
        *cache = Some(catalog.into_cache());
        let mut plan = plan?;
        plan.resolve_selected(&mut facts.types, &facts.aggregates)
            .map_err(error)?;
        let precise = precise_live_drops(report, facts).map_err(|message| error(&message))?;
        if plan.eligibility().next().is_some() || plan.active() {
            let mut contract = selected.take().unwrap_or_else(|| input.clone());
            body::defining_cleanups::select(
                facts,
                body,
                &mut contract,
                plan.eligibility(),
                precise,
                plan.active(),
            )
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

fn qualify_native(
    facts: &mut Declarations,
    input: &Contract,
    regions: &mut regions::Body,
    verifier: &borrow::incremental::Verifier,
    (cache, requirements): (
        &mut Option<borrow::destruction::glue::Cache>,
        Option<&body::drops::Requirements>,
    ),
    copies: &mut Copies,
    initial: &mut borrow::incremental::ConstInitial<'_>,
) -> std::result::Result<Option<Contract>, borrow::Error> {
    let body = initial.body();
    if !function_qualification_body(facts, body) {
        return Ok(None);
    }
    let error = |message: String| borrow::Error {
        node: Some(body.source.into()),
        message,
    };
    let recipes = cache
        .take()
        .ok_or_else(|| error("const qualification lost its recipe cache".into()))?;
    let catalog = recipes.attach(&facts.types, &facts.aggregates, &facts.members);
    let result = with_environment(facts, copies, |environment| {
        verifier.begin_const_qualification(environment, initial, regions, (&catalog, requirements))
    });
    *cache = Some(catalog.into_cache());
    result?;
    let Some(questions) = initial
        .questions(&mut facts.types, &facts.aggregates)
        .map_err(|message| error(message.into()))?
    else {
        return Ok(None);
    };
    let mut input = input.clone();
    let answers =
        body::defining_cleanups::qualify(facts, body, &mut input, questions).map_err(error)?;
    *regions = recheck(facts, body, &input).map_err(error)?;
    with_environment(facts, copies, |environment| {
        verifier.accept_const_qualification(environment, initial, regions, &answers)
    })?;
    Ok(Some(input))
}
/// Factories and other body owners retain their existing sealed proof paths.
/// Skipping a new function-only query does not pay active cleanup eligibility.
fn function_qualification_body(facts: &Declarations, body: &Body) -> bool {
    matches!(body.owner, body::ir::Owner::Function {definition, execution: None, ..}
        if facts.members.constructor(definition).is_none())
}
fn precise_live_drops(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
) -> Result<bool> {
    if let Some(catalog) = facts.source_bindings.as_deref() {
        catalog.require_provider_report(report)?;
    }
    Ok(report.package.requested_const_precise_live_drops()?)
}
