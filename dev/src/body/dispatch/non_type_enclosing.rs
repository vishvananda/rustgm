//! Source body identity selects its own completed feature scope before concrete dispatch.
use super::{Body, Contract, Declarations};
use crate::{
    body::ir::Owner, sema::dispatch::Dispatcher, types::parameters::environment::Environment,
};
use std::{cell::Cell, sync::Arc};

pub(super) fn prepare(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    first: u32,
) -> Result<Dispatcher, String> {
    let environment = environment(facts, body)?;
    let Some(environment) = environment.filter(|environment| environment.has_feature_assumptions())
    else {
        return Dispatcher::new(contract.universals, first).map_err(str::to_owned);
    };
    if contract.key != body.key() || contract.source != body.source {
        return Err("feature dispatch contract belongs to another body".into());
    }
    if body.source.source().is_some() {
        body.owner
            .source_caller(facts, body.source, body.instance)?;
    }
    match body.owner {
        Owner::Function { definition, .. }
            if body.source.source().is_some()
                && !facts.typed_function_bodies.contains(&definition) =>
        {
            return Err("feature dispatch requires its completed source body".into());
        }
        Owner::Initializer { definition, .. }
        | Owner::InitializerInstance {
            instance: crate::sema::values::Instance { definition, .. },
            ..
        } if !facts
            .initializer_index(definition)
            .and_then(|index| facts.initializers.get(index))
            .is_some_and(crate::sema::model::Initializer::type_checked) =>
        {
            return Err("feature dispatch requires its checked initializer".into());
        }
        _ => {}
    }
    let work = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let base = facts.constant_work.template_bytes;
    let result = Dispatcher::in_body(
        &facts.types,
        contract.universals,
        first,
        environment,
        &|steps, allocation| {
            work.set(work.get().saturating_add(steps));
            bytes.set(bytes.get().saturating_add(allocation));
            if work.get() > 1_000_000 || base.saturating_add(bytes.get()) > 64 * 1024 * 1024 {
                return Err("concrete feature dispatch work or storage budget exceeded");
            }
            Ok(())
        },
    );
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work.get());
    result
}
fn environment(facts: &mut Declarations, body: &Body) -> Result<Option<Arc<Environment>>, String> {
    let Some(source) = body.source.source() else {
        return imported(facts, body);
    };
    let owner = match body.owner {
        Owner::Function { definition, .. }
        | Owner::Initializer { definition, .. }
        | Owner::InitializerInstance {
            instance: crate::sema::values::Instance { definition, .. },
            ..
        } => Some(definition),
        Owner::Closure { identity, .. } => facts
            .closures
            .identity(identity)
            .and_then(|closure| closure.owner),
        Owner::AnonymousConstant | Owner::AnonymousInstance(_) => facts
            .inline_constants
            .as_deref()
            .and_then(|registry| registry.get(source))
            .and_then(|constant| constant.declaration),
        Owner::ClosureAdapter { .. } => None,
    };
    Ok(owner
        .and_then(|owner| facts.generics.body_environment(owner))
        .cloned())
}
fn imported(facts: &mut Declarations, body: &Body) -> Result<Option<Arc<Environment>>, String> {
    let Owner::Function {
        definition,
        signature,
        execution,
    } = body.owner
    else {
        return Ok(None);
    };
    let mut steps = 1_usize;
    let result = (|| {
        let header = facts
            .function_header(definition)?
            .ok_or("imported dispatch omitted its actual installed function header")?;
        let Some(environment) = header
            .environment
            .filter(|env| env.has_feature_assumptions())
        else {
            return Ok(None);
        };
        let original = crate::sema::crates::source_original(facts, definition, &mut steps)?
            .ok_or("imported feature dispatch requires its actual source original")?;
        if !original.rows().descriptor().profile().feature_predicates()
            || original.definition() != definition
            || original.signature() != signature
            || original.source()? != body.source
            || header.body != Some(body.source)
            || original.universals() != 0
            || execution.is_some_and(|actual| actual != signature)
            || !Arc::ptr_eq(environment, original.environment()?)
            || body.instance.is_some_and(|instance| {
                !matches!(facts.types.kind(instance.ty()),
                Some(crate::types::Kind::FunctionItem { item, signature: actual, .. })
                    if item.0 == definition.0 && actual == signature)
            })
        {
            return Err(
                "imported feature dispatch changed its installed body/header/instance scope",
            );
        }
        Ok(Some(environment.clone()))
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(steps);
    result.map_err(str::to_owned)
}

#[cfg(test)]
#[path = "../../../tests/invariants/dispatching_features.rs"]
mod tests;
