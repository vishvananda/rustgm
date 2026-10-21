//! Generic bodies share ordinary proofs with scoped moves and abstract cleanup.
use super::Analysis;
use crate::{
    body::templates::Templates,
    borrow::{
        Error,
        destruction::symbolic,
        ownership::{BodyFacts, initialization},
        regions, storage,
    },
    types::{
        Store,
        copy::Copies,
        destruction::{NeedsDrop, regions::Regions},
        outlives::LiveRegions,
    },
};
use std::collections::HashMap;

/// Prove moves, initialization, abstract destruction and loans in every template.
/// Provenance and promotion must already be checked on these retained bodies.
/// This result authorizes no specialization or native execution.
/// # Errors
/// Rejects missing contracts, invalid moves, destructor/loan conflicts and budgets.
pub fn verify<S: std::hash::BuildHasher>(
    environment: &mut crate::borrow::Environment<'_, S>,
    templates: &Templates,
    contracts: &regions::Analysis,
    custom: &[crate::types::ItemId],
    position: impl Fn(
        crate::types::TypeId,
        crate::body::ir::Selection,
    ) -> Result<(u32, u32), &'static str>,
) -> Result<Vec<Analysis>, Error> {
    let error = |message: &str| Error {
        node: None,
        message: message.into(),
    };
    if contracts.bodies.len() != templates.bodies.len() {
        return Err(error(
            "template cleanup contracts have different body coverage",
        ));
    }
    let mut index = HashMap::with_capacity(contracts.bodies.len());
    for contract in &contracts.bodies {
        if index.insert(contract.key, contract).is_some() {
            return Err(error("duplicate template cleanup contract"));
        }
    }
    let custom: std::collections::HashSet<_> = custom.iter().copied().collect();
    let needed = NeedsDrop::new(custom.iter().copied());
    let drops = Regions::new(custom.iter().copied()).map_err(error)?;
    let mut components = LiveRegions::default();
    let mut results = Vec::with_capacity(templates.bodies.len());
    for template in &templates.bodies {
        let body = &template.body;
        let error = |message: &str| Error {
            node: Some(body.source),
            message: message.into(),
        };
        validate(template).map_err(error)?;
        let contract = index
            .remove(&body.key())
            .ok_or_else(|| error("template cleanup contract is missing or repeated"))?;
        let parameters = crate::borrow::permissions::check(environment, contract)?
            .unwrap_or_else(|| std::sync::Arc::clone(&template.environment));
        let mut copies = Copies::in_environment(std::sync::Arc::clone(&parameters));
        let facts = initialization::analyze(
            environment.types,
            environment.definitions,
            environment.derives,
            &mut copies,
            body,
        )
        .map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?;
        let analysis = if no_cleanup(&template.destruction) {
            // Scoped requirements prove the absence of destruction. Moves from
            // custom-Drop containers still need their independent legality check.
            facts
                .destructor_moves(environment.types, |item| custom.contains(&item))
                .map_err(|e| Error {
                    node: Some(e.node),
                    message: e.message.into(),
                })?;
            super::check(
                environment.types,
                &mut components,
                body,
                contract,
                &facts.ownership,
                &facts.storage,
            )
        } else {
            let mut needed = needed.scoped(std::sync::Arc::clone(&parameters));
            let mut drops = drops.scoped(parameters);
            let plan = symbolic::select(
                environment.types,
                environment.definitions,
                &mut needed,
                &facts,
                |item| custom.contains(&item),
                &position,
            )
            .map_err(|e| Error {
                node: Some(e.node),
                message: e.message.into(),
            })?;
            super::check_schedule(
                environment.types,
                environment.definitions,
                &mut components,
                &mut drops,
                &facts,
                contract,
                plan.schedule(),
            )
        };
        results.push(analysis.map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?);
    }
    Ok(results)
}

fn no_cleanup(requirements: &crate::body::drops::Requirements) -> bool {
    !requirements.locals.contains(&true)
        && !requirements.values.contains(&true)
        && !requirements.projections.contains(&true)
        && requirements.sites.is_empty()
}

fn validate(template: &crate::body::templates::Template) -> Result<(), &'static str> {
    let body = &template.body;
    if template.destruction.locals.len() != body.locals.len()
        || template.destruction.values.len() != body.values.len()
        || template.destruction.projections.len() != body.projections.len()
    {
        return Err("template loan destruction facts have different body coverage");
    }
    Ok(())
}

/// Check each selected template using its scoped moves and proved provenance.
/// Possible destruction still requires the separate abstract cleanup timeline.
/// The result cannot authorize concrete specialization or execution.
/// # Errors
/// Rejects incomplete/foreign facts, pending cleanup, loan conflicts and budgets.
pub fn check(
    types: &Store,
    components: &mut LiveRegions,
    templates: &Templates,
    contracts: &regions::Analysis,
    ownership: &[BodyFacts],
) -> Result<Vec<Analysis>, Error> {
    if ownership.len() != templates.bodies.len() || contracts.bodies.len() != templates.bodies.len()
    {
        return Err(Error {
            node: templates.bodies.first().map(|t| t.body.source),
            message: "template loan facts have different body coverage".into(),
        });
    }
    let mut index = HashMap::with_capacity(contracts.bodies.len());
    for contract in &contracts.bodies {
        if index.insert(contract.key, contract).is_some() {
            return Err(Error {
                node: Some(contract.source),
                message: "duplicate template loan contract".into(),
            });
        }
    }
    let mut results = Vec::with_capacity(templates.bodies.len());
    for (ordinal, template) in templates.bodies.iter().enumerate() {
        let body = &template.body;
        let error = |message: &str| Error {
            node: Some(body.source),
            message: message.into(),
        };
        validate(template).map_err(error)?;
        if template.destruction.locals.contains(&true)
            || template.destruction.values.contains(&true)
            || template
                .destruction
                .sites
                .iter()
                .any(|site| matches!(site.trigger, crate::body::drops::Trigger::Store(_)))
        {
            return Err(error(
                "generic loan checking requires abstract cleanup for possible destruction",
            ));
        }
        let contract = index
            .remove(&body.key())
            .ok_or_else(|| error("template loan contract is missing or repeated"))?;
        let storage = storage::check(body).map_err(|e| Error {
            node: Some(e.node),
            message: e.message.into(),
        })?;
        results.push(
            super::check(
                types,
                components,
                body,
                contract,
                &ownership[ordinal],
                &storage,
            )
            .map_err(|e| Error {
                node: Some(e.node),
                message: e.message.into(),
            })?,
        );
    }
    Ok(results)
}
