//! Context parents complete original proofs without becoming runtime candidates.
use super::{
    Checked, Contract, Contracts, Environment, Error, HashMap, State, Templates, loans, owners,
    regions,
};
use crate::types::{ItemId, TypeId, variance::Variances};
use std::{collections::HashSet, hash::BuildHasher};
pub(crate) struct Contexts {
    pub bodies: Templates,
    // Exact current source rows displaced by provider-added contracts. They
    // must also prove, but only for the indexed retained components.
    pub source: Vec<Contract>,
    pub imported: Option<Box<crate::body::copy_requirements::ImportedInputs>>,
}
impl Default for Contexts {
    fn default() -> Self {
        Self {
            bodies: Templates::source(Vec::new()),
            source: Vec::new(),
            imported: None,
        }
    }
}
pub(crate) fn verify<'a, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    variances: &Variances,
    contracts: &Contracts<'_>,
    templates: &'a Templates,
    context: &'a Contexts,
    custom: &[ItemId],
    position: impl Fn(TypeId, crate::body::ir::Selection) -> Result<(u32, u32), &'static str>,
) -> Result<Checked<'a>, Error> {
    if context.bodies.bodies.is_empty() && context.source.is_empty() && context.imported.is_none() {
        return super::verify(
            environment,
            variances,
            contracts,
            templates,
            custom,
            position,
        );
    }
    let contexts = &context.bodies;
    let count = templates
        .len()
        .checked_add(contexts.bodies.len())
        .ok_or_else(|| error("original context count overflow"))?;
    check_budget(contracts, templates, context, count)?;
    let owners::Indices {
        functions: index,
        closures,
        anonymous,
    } = owners::indices(templates)?;
    owners::indices(contexts)?;
    let inputs = context_inputs(contracts, templates, contexts)?;
    let mut overlaid = None;
    if let Some(imported) = &context.imported {
        let mut proof_inputs = inputs.clone();
        for prepared in imported
            .inputs(environment.types, templates.imported_originals())
            .map_err(error)?
        {
            let target = proof_inputs
                .get_mut(&prepared.key)
                .ok_or_else(|| error("prepared imported input lost its raw original"))?;
            if target.source != prepared.source || target.signature != prepared.signature {
                return Err(error("prepared imported input changed its raw header"));
            }
            *target = prepared;
        }
        overlaid = Some(proof_inputs);
    }
    let proof_inputs = overlaid.as_ref().unwrap_or(&inputs);
    source_regions(environment, variances, contracts, templates, context)?;
    let mut regions = regions::check_nested_bodies(
        environment.types,
        variances,
        contracts,
        templates.iter().chain(&contexts.bodies).map(|template| {
            proof_inputs
                .get(&template.body.key())
                .map(|&input| (input, &template.body))
                .ok_or_else(|| regions::Error {
                    source: Some(template.body.source.into()),
                    message: "original context requires its exact input contract".into(),
                })
        }),
    )
    .map_err(|e| Error {
        node: e.source,
        message: e.message,
    })?;
    let mut loans = loans::templates::verify_iter(
        environment,
        templates.iter().chain(&contexts.bodies),
        count,
        &regions,
        custom,
        position,
    )?;
    // Context rows paid the original component, but remain absent from every
    // checked lookup and observed executable-template row.
    regions.bodies.truncate(templates.len());
    loans.truncate(templates.len());
    let contracts = templates
        .iter()
        .map(|template| {
            inputs
                .get(&template.body.key())
                .copied()
                .cloned()
                .ok_or_else(|| error("checked template lost its original context contract"))
        })
        .collect::<Result<Vec<Contract>, Error>>()?;
    Ok(Checked {
        templates,
        state: State {
            index,
            closures,
            anonymous,
            regions,
            loans,
            contracts,
        },
    })
}
fn check_budget(
    contracts: &Contracts<'_>,
    templates: &Templates,
    context: &Contexts,
    count: usize,
) -> Result<(), Error> {
    let clone_bytes = if context.imported.is_some() {
        templates
            .inputs(&contracts.bodies)
            .map(Contract::retained_bytes)
            .fold(0, usize::saturating_add)
    } else {
        0
    };
    if crate::support::retained::new_map::<crate::body::ir::BodyKey, usize>(count)
        .saturating_mul(2)
        .saturating_add(crate::support::retained::new_map::<
            crate::body::ir::NodeKey,
            (),
        >(count))
        .saturating_add(
            crate::support::retained::new_map::<crate::body::ir::BodyKey, &Contract>(
                templates.input_count(contracts.bodies.len()),
            )
            .saturating_mul(if context.imported.is_some() { 2 } else { 1 }),
        )
        .saturating_add(crate::support::retained::new_map::<
            crate::body::ir::BodyKey,
            (),
        >(context.source.len()))
        .saturating_add(clone_bytes)
        .saturating_add(
            context
                .imported
                .as_ref()
                .map_or(0, |inputs| inputs.combined_bytes()),
        )
        > 64 * 1024 * 1024
    {
        return Err(error("original proof contexts exceed storage budget"));
    }
    Ok(())
}
fn context_inputs<'a>(
    contracts: &'a Contracts<'_>,
    templates: &'a Templates,
    contexts: &Templates,
) -> Result<HashMap<crate::body::ir::BodyKey, &'a Contract>, Error> {
    let mut keys: HashSet<_> = templates.iter().map(|t| t.body.key()).collect();
    let mut sources: HashSet<_> = templates.iter().map(|t| t.body.source).collect();
    for template in &contexts.bodies {
        if !keys.insert(template.body.key()) || !sources.insert(template.body.source) {
            return Err(error("duplicate original context owner or source"));
        }
    }
    let mut inputs = HashMap::with_capacity_and_hasher(
        templates.input_count(contracts.bodies.len()),
        crate::support::hash::BuildFxHasher::default(),
    );
    for contract in templates.inputs(&contracts.bodies) {
        if inputs.insert(contract.key, contract).is_some() {
            return Err(error("duplicate original context input contract"));
        }
    }
    Ok(inputs)
}
fn source_regions<S: BuildHasher>(
    environment: &Environment<'_, S>,
    variances: &Variances,
    contracts: &Contracts<'_>,
    templates: &Templates,
    context: &Contexts,
) -> Result<(), Error> {
    if context.source.is_empty() {
        return Ok(());
    }
    let bodies: HashMap<_, _> = templates
        .iter()
        .chain(&context.bodies.bodies)
        .map(|t| (t.body.key(), &t.body))
        .collect();
    let mut keys = HashSet::with_capacity(context.source.len());
    regions::check_nested_bodies(
        environment.types,
        variances,
        contracts,
        context.source.iter().map(|input| {
            if !keys.insert(input.key) {
                return Err(regions::Error {
                    source: Some(input.source.into()),
                    message: "duplicate retained source contract".into(),
                });
            }
            bodies
                .get(&input.key)
                .copied()
                .map(|body| (input, body))
                .ok_or_else(|| regions::Error {
                    source: Some(input.source.into()),
                    message: "retained source contract lost its exact component body".into(),
                })
        }),
    )
    .map_err(|e| Error {
        node: e.source,
        message: e.message,
    })?;
    Ok(())
}
fn error(message: &str) -> Error {
    Error {
        node: None,
        message: message.into(),
    }
}
