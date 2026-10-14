//! Generic CFGs retain possible destruction; they are not executable programs.
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
/// Build each type/const generic body once using the ordinary scope/CFG builder.
/// The result retains possible cleanup even without a declared custom destructor.
/// It cannot be passed to the concrete program verifier or `LowIR` lowerer.
/// # Errors
/// Reports missing typed facts, unsupported operations and exhausted work.
pub fn build(report: &Report, facts: &Declarations) -> Result<Templates, Error> {
    let mut scratch = Scratch::new(facts);
    let query = NeedsDrop::new(facts.destructors.implementations().iter().map(|i| i.item));
    let mut bodies = Vec::new();
    for function in &facts.functions {
        if !facts.generics.has_parameters(function.definition) {
            continue;
        }
        let environment = facts
            .generics
            .body_environment(function.definition)
            .ok_or_else(|| {
                Declarations::error(
                    report,
                    function.node,
                    "missing completed generic body environment",
                )
            })?;
        let mut scoped = query.scoped(Arc::clone(environment));
        let body = function_body(report, facts, &mut scratch, function, Some(&mut scoped))?;
        let destruction = crate::body::drops::requirements(facts, &body, &mut scoped)
            .map_err(|message| Declarations::error(report, body.source, message))?;
        bodies.push(Template {
            body,
            destruction,
            environment: Arc::clone(environment),
        });
    }
    Ok(Templates { bodies })
}
