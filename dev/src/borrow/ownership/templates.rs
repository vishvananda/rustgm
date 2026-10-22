//! Moves in unused generic bodies are checked under their fixed assumptions.
use super::{BodyFacts, Error, analyze_body, destructors};
use crate::{
    body::templates::Templates,
    types::{
        ItemId, Store,
        aggregate::Definitions,
        copy::{Copies, Declaration},
    },
};
use std::{hash::BuildHasher, sync::Arc};

/// Reuse the ordinary move/initialization proof without granting an executable
/// token. Generic cleanup, loans and specialization still require separate proof.
/// # Errors
/// Reports invalid moves/initialization, destructor field moves and work limits.
pub fn analyze<S: BuildHasher>(
    types: &Store,
    definitions: &Definitions,
    derives: &std::collections::HashMap<ItemId, Declaration, S>,
    custom: Option<&impl Fn(ItemId) -> bool>,
    templates: &Templates,
) -> Result<Vec<BodyFacts>, Error> {
    let mut result = Vec::with_capacity(templates.bodies.len());
    for template in &templates.bodies {
        let mut copies = Copies::in_environment(Arc::clone(&template.environment));
        let proof = analyze_body(types, definitions, derives, &mut copies, &template.body)?;
        if let Some(custom) = custom {
            destructors::check(types, custom, &template.body, &proof)?;
        }
        result.push(proof);
    }
    Ok(result)
}
