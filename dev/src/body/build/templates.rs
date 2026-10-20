//! Definition CFGs retain possible destruction; they are not executable programs.
pub use super::constructors::templates as constructors;
use super::{Declarations, Error, Report, Scratch, function_body};
use crate::{
    body::{drops::Requirements, ir::Body},
    types::{destruction::NeedsDrop, parameters::environment::Environment},
};
use std::sync::Arc;

pub struct Template {
    pub body: Body,
    pub destruction: Requirements,
    pub environment: Arc<Environment>,
}
pub struct Templates {
    pub bodies: Vec<Template>,
}
/// Build each generic or opaque-defining body once using the ordinary scope/CFG builder.
/// The result retains possible cleanup even without a declared custom destructor.
/// It cannot be passed to the concrete program verifier or `LowIR` lowerer.
/// # Errors
/// Reports missing typed facts, unsupported operations and exhausted work.
pub fn build(report: &Report, facts: &Declarations) -> Result<Templates, Error> {
    let mut scratch = Scratch::new(facts);
    let query = NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item));
    let mut bodies = Vec::new();
    for function in &facts.functions {
        if function.body.is_none()
            || (!facts.generics.has_parameters(function.definition)
                && !facts.opaques.owners.contains_key(&function.definition))
        {
            continue;
        }
        let environment =
            if let Some(environment) = facts.generics.body_environment(function.definition) {
                Arc::clone(environment)
            } else {
                Arc::new(
                    facts
                        .generics
                        .parameter_environment(&facts.types, function.definition)
                        .map_err(|e| Declarations::error(report, function.node, e))?,
                )
            };
        let mut scoped = query.scoped(Arc::clone(&environment));
        let body = function_body(report, facts, &mut scratch, function, Some(&mut scoped))?;
        let destruction = crate::body::drops::requirements(facts, &body, &mut scoped)
            .map_err(|message| Declarations::error(report, body.source, message))?;
        bodies.push(Template {
            body,
            destruction,
            environment,
        });
    }
    for initializer in &facts.initializers {
        if facts.associated.owner(initializer.definition).is_none()
            && !facts.generics.has_parameters(initializer.definition)
        {
            continue;
        }
        let environment =
            if let Some(environment) = facts.generics.body_environment(initializer.definition) {
                Arc::clone(environment)
            } else {
                Arc::new(
                    facts
                        .generics
                        .parameter_environment(&facts.types, initializer.definition)
                        .map_err(|e| Declarations::error(report, initializer.value, e))?,
                )
            };
        let mut scoped = query.scoped(Arc::clone(&environment));
        let body =
            super::initializer_body(report, facts, &mut scratch, initializer, Some(&mut scoped))?;
        crate::body::constants::eligibility(report, facts, &body)?;
        let destruction = crate::body::drops::requirements(facts, &body, &mut scoped)
            .map_err(|message| Declarations::error(report, body.source, message))?;
        bodies.push(Template {
            body,
            destruction,
            environment,
        });
    }
    crate::body::constants::alias_cycles(report, facts, &bodies)?;
    Ok(Templates { bodies })
}
