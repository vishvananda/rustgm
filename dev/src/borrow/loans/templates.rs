//! Generic loans without possible cleanup use the ordinary liveness/dataflow proof.
use super::Analysis;
use crate::{
    body::templates::Templates,
    borrow::{Error, ownership::BodyFacts, regions, storage},
    types::{Store, outlives::LiveRegions},
};
use std::collections::HashMap;

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
        if index.insert(contract.source, contract).is_some() {
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
        if template.destruction.locals.len() != body.locals.len()
            || template.destruction.values.len() != body.values.len()
            || template.destruction.projections.len() != body.projections.len()
        {
            return Err(error(
                "template loan destruction facts have different body coverage",
            ));
        }
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
            .remove(&body.source)
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
