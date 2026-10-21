//! Typed inline declarations and completed roots retain source identity across bodies.
use super::graph::Graph;
use crate::sema::inline_constants::{Body, Evaluation};

pub(super) fn declarations<'a>(graph: &mut Graph<'a>) -> Result<Vec<&'a Body>, &'static str> {
    let Some(registry) = graph.facts.inline_constants.as_deref() else {
        return Ok(Vec::new());
    };
    let count = registry.bodies().count();
    graph.charge(count)?;
    graph.storage(count.saturating_mul(256 + size_of::<&Body>()))?;
    let mut bodies: Vec<_> = registry.bodies().collect();
    bodies.sort_unstable_by_key(|body| (body.source.file.0, body.source.node.0));
    for body in &bodies {
        let result = body
            .result
            .ok_or("constant view requires a completed inline result type")?;
        let status = match (body.evaluation, body.value.is_some()) {
            (Evaluation::Required, true) => "checked-value",
            (Evaluation::Required, false) => {
                return Err("constant view requires a completed inline value");
            }
            (Evaluation::DeferredTemplate, false) => "deferred-template",
            (Evaluation::DeferredUnreachable, false) => "deferred-unreachable",
            (Evaluation::SuppressedZeroRepeat, false) => "suppressed-zero-repeat",
            (_, true) => return Err("deferred inline constant unexpectedly retains a value"),
        };
        graph.declaration(format!(
            "declaration source={}:{} kind=inline type={} const=true status={status} parent={}:{}",
            body.source.file.0,
            body.source.node.0,
            result.0,
            body.parent_body.file.0,
            body.parent_body.node.0,
        ))?;
    }
    Ok(bodies)
}

pub(super) fn roots(graph: &mut Graph<'_>, bodies: Vec<&Body>) -> Result<(), &'static str> {
    for body in bodies {
        graph.charge(1)?;
        if body.evaluation != Evaluation::Required {
            continue;
        }
        let result = body
            .result
            .ok_or("inline root lacks a completed result type")?;
        let value = body.value.ok_or("inline root lacks its completed value")?;
        let value = graph.value(result, value)?;
        graph.root(format!(
            "root inline source={}:{} type={} value=v{value}",
            body.source.file.0, body.source.node.0, result.0,
        ))?;
    }
    Ok(())
}

pub(super) fn instances(graph: &mut super::graph::Graph<'_>) -> Result<(), &'static str> {
    let Some(registry) = graph.facts.inline_constants.as_deref() else {
        return Ok(());
    };
    let count = registry.instance_values().count();
    graph.charge(count)?;
    graph.storage(count.saturating_mul(96))?;
    let mut values = registry.instance_values().collect::<Vec<_>>();
    values.sort_unstable_by_key(|(instance, result, _)| (instance.order(), result.0));
    for (instance, result, value) in values {
        let value = graph.value(result, value)?;
        graph.root(format!(
            "root inline-instance instance={instance} type={} value=v{value}",
            result.0
        ))?;
    }
    Ok(())
}
