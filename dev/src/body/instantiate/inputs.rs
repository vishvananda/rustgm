//! Select exact callable arguments without rebuilding or inferring source bodies.
use super::{Bound, Declarations};
use crate::{
    body::ir::{Body, Owner},
    resolve::model::DefId,
    types::{Kind, TypeId, arguments::Argument, instance::Instance},
};
#[cfg(test)]
#[path = "../../../tests/invariants/public_signature_storage.rs"]
mod tests;

type Binding = (crate::types::parameters::ParameterId, Argument);

/// Retain the completed body's signature and prepare the distinct public roots.
/// Call only after actual namespace transport and projection normalization.
pub(super) fn finish_owner(
    facts: &mut Declarations,
    owner: &mut Owner,
    contract: Option<&crate::body::regions::Contract>,
) -> Result<(), &'static str> {
    let signature = contract.and_then(|input| input.signature);
    match owner {
        Owner::Function { execution, .. } => *execution = signature,
        Owner::Closure {
            execution: Some(execution),
            ..
        } => {
            *execution = signature.ok_or("captured execution requires its specialized contract")?;
        }
        _ => {}
    }
    prepare_signature(facts, *owner)
}

/// The selected public callable keeps its own binder, distinct from the body's
/// opened proof interface. Prepare its physical roots without substituting them
/// again or treating field readiness as lifetime or callable evidence.
pub(super) fn prepare_signature(
    facts: &mut Declarations,
    owner: Owner,
) -> Result<(), &'static str> {
    let Owner::Function { signature, .. } = owner else {
        return Ok(());
    };
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(signature)
    else {
        return Err("selected function owner has no public signature");
    };
    let count = facts
        .types
        .list(parameters)
        .ok_or("invalid public parameters")?
        .len();
    for index in 0..count {
        let ty = facts
            .types
            .list(parameters)
            .ok_or("invalid public parameters")?[index];
        facts.aggregates.prepare(&mut facts.types, ty)?;
    }
    facts.aggregates.prepare(&mut facts.types, result)
}

pub(super) struct Inputs {
    pub(super) caller: crate::sema::values::execution::CallerInstance,
    pub(super) instance: Option<Instance>,
    pub(super) bindings: Vec<Binding>,
    pub(super) captures: Vec<crate::types::RegionId>,
    pub(super) comparison: Option<(crate::types::TypeId, crate::types::TypeId)>,
    pub(super) connections: Option<crate::body::invocation_entries::SourceBindings>,
}
pub(super) fn select(facts: &mut Declarations, instance: Instance) -> Result<Inputs, &'static str> {
    if let Some(Kind::Closure { .. }) = facts.types.kind(instance.ty()) {
        return closure(facts, instance, false);
    }
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = facts.types.kind(instance.ty())
    else {
        return Err("invalid callable instance");
    };
    let Some((callable, parameters)) = facts.lifetimes.callables.registration(item) else {
        let owner = crate::resolve::model::DefId(item.0);
        if arguments != crate::types::GenericArgs::EMPTY
            || facts.generics.has_parameters(owner)
            || facts.definitions.get(item.0 as usize).copied().flatten() != Some(instance.ty())
        {
            return Err("missing callable template");
        }
        return Ok(Inputs {
            caller: crate::sema::values::execution::CallerInstance::Callable(instance),
            instance: Some(instance),
            bindings: Vec::new(),
            captures: Vec::new(),
            comparison: None,
            connections: None,
        });
    };
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("invalid callable arguments")?
        .collect::<Vec<_>>();
    let early = callable.parameters as usize;
    if arguments.len() != early.saturating_add(parameters.len()) {
        return Err("callable instance argument count differs from its template");
    }
    let mut regions = Vec::with_capacity(early);
    for argument in &arguments[..early] {
        let Argument::Lifetime(region) = *argument else {
            return Err("callable early argument is not a lifetime");
        };
        regions.push(region);
    }
    let bindings = parameters
        .iter()
        .copied()
        .zip(arguments[early..].iter().copied())
        .collect::<Vec<_>>();
    let mut selected = Bound::parameters_captured(&facts.types, &bindings, regions)?;
    let expected = selected.apply(&mut facts.types, callable.signature)?;
    let projected = facts.types.contains_projections(expected) == Some(true);
    if !projected && expected != signature {
        return Err("instance signature differs from substituted template");
    }
    Ok(Inputs {
        caller: crate::sema::values::execution::CallerInstance::Callable(instance),
        instance: Some(instance),
        bindings,
        captures: Vec::new(),
        comparison: projected.then_some((expected, signature)),
        connections: None,
    })
}

/// A lifetime-only declaration context is not an invocation with erased arguments.
/// Its exact anonymous Contract already retains the original slots and bounds.
pub(super) fn original_function_context(
    facts: &mut Declarations,
    instance: crate::sema::inline_constants::InlineInstance,
    original: &crate::body::regions::Contract,
) -> Result<Option<Inputs>, &'static str> {
    use crate::sema::inline_constants::{Context, InlineInstance};
    let Context::Callable(callable) = instance.enclosing() else {
        return Ok(None);
    };
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = facts.types.kind(callable.ty())
    else {
        return Ok(None);
    };
    if arguments != crate::types::GenericArgs::EMPTY {
        return Ok(None);
    }
    let owner = DefId(item.0);
    let Some(scheme) = facts.lifetimes.functions.get(&owner) else {
        return Ok(None);
    };
    if scheme.captures == 0 && scheme.early.is_empty() {
        return Ok(None);
    }
    let header = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("anonymous original callable lost its lifetime header")?;
    let function = facts
        .function_indices
        .get(&owner)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("anonymous original callable has no real function")?;
    let inline = facts
        .inline_constants
        .as_deref()
        .and_then(|registry| registry.get(instance.source()))
        .ok_or("anonymous original callable has no typed inline body")?;
    if facts.generics.has_parameters(owner)
        || facts.definitions.get(item.0 as usize).copied().flatten() != Some(callable.ty())
        || function.definition != owner
        || function.body != Some(inline.context_owner())
        || function.signature != signature
        || header.parameters as usize != scheme.captures as usize + scheme.early.len()
        || inline.universals != scheme.total
        || original.key != crate::body::ir::BodyKey::Anonymous(instance.source())
        || original.source != instance.source().into()
        || original.universals != inline.universals
        || InlineInstance::select(facts, instance.source(), instance.enclosing())? != instance
    {
        return Err("anonymous original callable context changed its declaration or binder");
    }
    let mut work = 0;
    let mapping = scheme.callable_mapping(&mut work);
    facts.body_work.canonical_steps += work;
    let mut relocation = Bound::new(mapping?);
    let late = u32::try_from(scheme.late.len()).map_err(|_| "too many callable late lifetimes")?;
    let expected = relocation.signature(&mut facts.types, signature, late);
    facts.body_work.canonical_steps += relocation.steps;
    if expected? != header.signature {
        return Err("anonymous original callable differs from its exact public relocation");
    }
    Ok(Some(Inputs {
        caller: crate::sema::values::execution::CallerInstance::Inline(instance),
        instance: None,
        bindings: Vec::new(),
        captures: Vec::new(),
        comparison: None,
        connections: None,
    }))
}

pub(super) fn closure(
    facts: &mut Declarations,
    instance: Instance,
    anonymous: bool,
) -> Result<Inputs, &'static str> {
    let Some(Kind::Closure {
        identity,
        signature,
        environment,
        ..
    }) = facts.types.kind(instance.ty())
    else {
        return Err("closure instance requires an implemented environment adapter");
    };
    let body = *facts
        .closures
        .identity(identity)
        .ok_or("missing original closure body")?;
    let mut inputs = super::closures::arguments(facts, instance, body)?;
    if environment != TypeId::UNIT {
        let original = crate::body::invocation_entries::validate_execution(&facts.types, body)?;
        let substituted = super::substitute_closure_execution(facts, instance, original)?;
        let formal = if anonymous {
            super::closure_self_type(facts, substituted)?
        } else {
            super::normalize_closure_execution(facts, substituted)?
        };
        let mut connections = crate::body::invocation_entries::source_bindings(
            &facts.types,
            formal.execution,
            instance,
            body.universals,
        )?;
        if anonymous {
            // The anonymous contract owns no receiver slots. No enclosing
            // conditional fact or undischarged Self debt becomes its permission.
            if !connections.equalities.is_empty()
                || formal.proof.as_ref().is_some_and(|proof| !proof.is_empty())
            {
                return Err(
                    "anonymous captured context validation requires lifetime proof transport",
                );
            }
        } else {
            connections.proof = formal.proof;
            inputs.connections = Some(connections);
        }
        return Ok(inputs);
    }
    let expected = inherited_signature(&mut facts.types, body, &inputs.captures, signature)?;
    let mut selected = Bound::parameters(&facts.types, &inputs.bindings)?;
    let expected = selected.apply(&mut facts.types, expected)?;
    let projected = facts.types.contains_projections(expected) == Some(true);
    if !projected && expected != signature {
        return Err("closure signature differs from substituted body template");
    }
    inputs.comparison = projected.then_some((expected, signature));
    Ok(inputs)
}

/// Reconstruct the source call binder from the body's distinct proof universe.
fn inherited_signature(
    types: &mut crate::types::Store,
    body: crate::sema::closures::Body,
    captures: &[crate::types::RegionId],
    actual: TypeId,
) -> Result<TypeId, &'static str> {
    use crate::types::RegionId;
    if captures.is_empty() {
        return Ok(body.signature);
    }
    let Some(Kind::Function { bound_regions, .. }) = types.kind(actual) else {
        return Err("closure instance has no call signature");
    };
    if body.universals.checked_sub(body.inherited) != Some(bound_regions) {
        return Err("closure signature lifetime interface differs from its body");
    }
    let mut mapping = crate::support::hash::HashMap::default();
    for (index, &capture) in captures.iter().enumerate() {
        let index = u16::try_from(index).map_err(|_| "too many inherited closure lifetimes")?;
        let capture = match capture.bound_parts() {
            Some((depth, slot)) => RegionId::bound(
                depth
                    .checked_add(1)
                    .ok_or("closure lifetime binder depth exhausted")?,
                slot,
            )?,
            None => capture,
        };
        mapping.insert(RegionId::bound(0, index)?, capture);
    }
    for index in 0..bound_regions {
        let source =
            u16::try_from(body.inherited + index).map_err(|_| "too many closure body lifetimes")?;
        let target = u16::try_from(index).map_err(|_| "too many closure call lifetimes")?;
        mapping.insert(RegionId::bound(0, source)?, RegionId::bound(0, target)?);
    }
    Bound::restore_regions(mapping).signature(types, body.signature, bound_regions)
}

pub(super) fn owner(
    facts: &Declarations,
    source: &Body,
    instance: Instance,
) -> Result<(Owner, DefId), &'static str> {
    if source.instance.is_some() {
        return Err("instance does not select an original body template");
    }
    match (source.owner, facts.types.kind(instance.ty())) {
        (
            Owner::Function {
                definition,
                execution: None,
                ..
            },
            Some(Kind::FunctionItem {
                item, signature, ..
            }),
        ) if definition.0 == item.0 => Ok((
            Owner::Function {
                definition,
                signature,
                execution: None,
            },
            definition,
        )),
        (
            Owner::Closure {
                identity,
                execution,
                ..
            },
            Some(Kind::Closure {
                identity: selected,
                signature,
                ..
            }),
        ) if identity == selected => {
            let closure = facts
                .closures
                .identity(identity)
                .ok_or("missing closure template identity")?;
            if crate::body::ir::BodyOrigin::from(closure.semantic_source()) != source.source {
                return Err("closure instance selects a different source body");
            }
            let owner = closure
                .owner
                .ok_or("missing generic closure environment owner")?;
            Ok((
                Owner::Closure {
                    identity,
                    signature,
                    execution,
                },
                owner,
            ))
        }
        _ => Err("instance does not select its original body owner"),
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/original_contexts.rs"]
mod original_context_tests;

#[cfg(test)]
#[path = "../../../tests/invariants/mono_imports.rs"]
mod monomorphic_import_tests;
