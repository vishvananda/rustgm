//! Source declarations precede promotions in stable owner/specialization/value order.
use super::graph::Graph;
use crate::{
    body::ir::{BodyKey, Owner},
    borrow::Checked,
    sema::model::InitializerKind,
};

pub(super) fn collect(graph: &mut Graph<'_>, checked: &Checked<'_>) -> Result<(), &'static str> {
    declarations(graph)?;
    let inline = super::inline::declarations(graph)?;
    initializers(graph)?;
    associated(graph)?;
    super::inline::roots(graph, inline)?;
    super::inline::instances(graph)?;
    promotions(graph, checked)
}
fn declarations(graph: &mut Graph<'_>) -> Result<(), &'static str> {
    let facts = graph.facts;
    let count = facts
        .functions
        .len()
        .checked_add(facts.initializers.len())
        .ok_or("constant declaration count overflow")?;
    graph.charge(count)?;
    graph.storage(
        count
            .checked_mul(256)
            .ok_or("constant declaration size overflow")?,
    )?;
    let mut declarations = Vec::with_capacity(count);
    for function in &facts.functions {
        let deferred = facts.generics.has_parameters(function.definition);
        declarations.push((
            function.definition.0,
            format!(
                "declaration def={} kind=function type={} const={} status={}",
                function.definition.0,
                function.signature.0,
                facts.is_const_function(function.definition),
                if deferred {
                    "deferred-template"
                } else if function.body.is_none() {
                    "declaration-only"
                } else {
                    "checked-body"
                }
            ),
        ));
    }
    for initializer in &facts.initializers {
        let kind = match initializer.kind {
            InitializerKind::Const => "const",
            InitializerKind::Static { mutable: false } => "static",
            InitializerKind::Static { mutable: true } => "static-mut",
        };
        let deferred = facts.associated.owner(initializer.definition).is_some()
            || facts.generics.has_parameters(initializer.definition);
        declarations.push((
            initializer.definition.0,
            format!(
                "declaration def={} kind={kind} type={} const=true status={}",
                initializer.definition.0,
                initializer.ty.0,
                if deferred {
                    "deferred-initializer"
                } else {
                    "checked-value"
                }
            ),
        ));
    }
    declarations.sort_unstable_by_key(|(id, _)| *id);
    for (_, row) in declarations {
        graph.declaration(row)?;
    }
    Ok(())
}
fn initializers(graph: &mut Graph<'_>) -> Result<(), &'static str> {
    let facts = graph.facts;
    let mut initializers: Vec<_> = facts.initializers.iter().collect();
    initializers.sort_unstable_by_key(|i| i.definition.0);
    for initializer in initializers {
        if facts.associated.owner(initializer.definition).is_some()
            || facts.generics.has_parameters(initializer.definition)
        {
            continue;
        }
        let value = graph
            .reader
            .definition(initializer.definition)
            .ok_or("constant view requires a completed initializer")?;
        let value_id = graph.value(initializer.ty, value)?;
        let storage = if matches!(initializer.kind, InitializerKind::Static { .. }) {
            format!(
                "a{}",
                graph.static_(initializer.definition, initializer.ty, Some(value))?
            )
        } else {
            "none".into()
        };
        graph.root(format!(
            "root initializer def={} type={} value=v{value_id} storage={storage}",
            initializer.definition.0, initializer.ty.0
        ))?;
    }
    Ok(())
}
fn promotions(graph: &mut Graph<'_>, checked: &Checked<'_>) -> Result<(), &'static str> {
    graph.charge(checked.program().bodies.len())?;
    graph.storage(checked.program().bodies.len().saturating_mul(16))?;
    let mut bodies: Vec<_> = checked.program().bodies.iter().collect();
    bodies.sort_unstable_by_key(|body| (owner_key(body.key()), body.instance.map(|i| i.ty().0)));
    for body in bodies {
        let Some(promotions) = &body.promotions else {
            continue;
        };
        let count = promotions.entries().count();
        graph.charge(count.saturating_mul(2))?;
        graph.storage(count.saturating_mul(32))?;
        let mut entries: Vec<_> = promotions.entries().collect();
        entries.sort_unstable_by_key(|(id, _)| id.0);
        let owner = match body.owner {
            Owner::ClosureAdapter { adapter, .. } => format!(
                "closure-adapter:{}:{}:{}",
                adapter.instance().ty().0,
                adapter.receiver().0,
                adapter.requested().name()
            ),
            Owner::Function { definition, .. } | Owner::Initializer { definition, .. } => {
                format!("def:{}", definition.0)
            }
            Owner::Closure { identity, .. } => format!("closure:{}", identity.0),
            Owner::InitializerInstance { instance, .. } => format!(
                "initializer-instance:{}:{}",
                instance.definition.0, instance.arguments.0
            ),
            Owner::AnonymousInstance(instance) => format!("anonymous-instance:{instance}"),
            Owner::AnonymousConstant => {
                format!("anonymous:{}:{}", body.source.file.0, body.source.node.0)
            }
        };
        for (id, value) in entries {
            let ty = body
                .values
                .get(id.0 as usize)
                .ok_or("constant promotion lacks a typed value")?
                .ty;
            let value = graph.value(ty, value)?;
            graph.root(format!(
                "root promotion owner={owner} instance={} expression={} type={} value=v{value}",
                body.instance
                    .map_or_else(|| "none".into(), |i| i.ty().0.to_string()),
                id.0,
                ty.0
            ))?;
        }
    }
    Ok(())
}

fn associated(graph: &mut Graph<'_>) -> Result<(), &'static str> {
    let count = graph.reader.associated_values().count();
    graph.charge(count)?;
    graph.storage(count.saturating_mul(128))?;
    let mut values = graph.reader.associated_values().collect::<Vec<_>>();
    values.sort_unstable_by_key(|(instance, _, _)| (instance.definition.0, instance.arguments.0));
    for (instance, ty, value) in values {
        let value = graph.value(ty, value)?;
        graph.root(format!(
            "root associated-instance instance={}:{} type={} value=v{value}",
            instance.definition.0, instance.arguments.0, ty.0
        ))?;
    }
    Ok(())
}

fn owner_key(key: BodyKey) -> (u8, u32, u32, u8, u32, u32) {
    match key {
        BodyKey::ClosureAdapter(adapter) => (
            3,
            adapter.instance().ty().0,
            adapter.requested() as u32,
            0,
            adapter.receiver().0,
            0,
        ),
        BodyKey::InitializerInstance(instance) => {
            (5, instance.definition.0, instance.arguments.0, 0, 0, 0)
        }
        BodyKey::Declaration(id) => (0, id.0, 0, 0, 0, 0),
        BodyKey::Closure(id) => (1, id.0, 0, 0, 0, 0),
        BodyKey::Anonymous(node) => (2, node.file.0, node.node.0, 0, 0, 0),
        BodyKey::AnonymousInstance(instance) => {
            let (file, node, kind, owner, args) = instance.order();
            (4, file, node, kind, owner, args)
        }
    }
}
