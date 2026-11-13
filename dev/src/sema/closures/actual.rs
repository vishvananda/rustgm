//! Selected closure arguments retain the original source binder before substitution.
use super::{Body, Execution};
use crate::{
    sema::model::Declarations,
    types::{
        ItemId, Kind, RegionId, arguments::Argument, instance::Instance, parameters::ParameterId,
        substitute::Bound,
    },
};
use std::borrow::Cow;
pub(crate) struct Arguments {
    pub bindings: Vec<(ParameterId, Argument)>,
    pub captures: Vec<RegionId>,
}

/// Instantiation changes arguments, never the source closure's public binder.
pub(crate) fn call_binders(facts: &Declarations, source: Body) -> Result<u32, &'static str> {
    let Some(Kind::Closure {
        identity,
        signature,
        ..
    }) = facts
        .node(source.source)
        .and_then(|node| node.ty)
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err("closure source has no retained public type");
    };
    if identity != source.identity {
        return Err("closure public type belongs to a different source");
    }
    let Some(Kind::Function { bound_regions, .. }) = facts.types.kind(signature) else {
        return Err("closure source has no retained call binder");
    };
    Ok(bound_regions)
}

/// A defining closure can precede its enclosing opaque callable's template.
/// This is the producer's declaration order, not executable or caller evidence.
pub(crate) fn defining_parameters(
    facts: &Declarations,
    owner: crate::resolve::model::DefId,
) -> Result<Cow<'_, [ParameterId]>, &'static str> {
    match facts
        .lifetimes
        .callables
        .generic_parameters(ItemId(owner.0))
    {
        Some(parameters) => Ok(Cow::Borrowed(parameters)),
        None => facts
            .generics
            .callable_parameters(&facts.types, owner)
            .map(|(_, parameters)| Cow::Owned(parameters)),
    }
}

pub(crate) fn defining_arguments(
    facts: &Declarations,
    instance: Instance,
    body: Body,
) -> Result<Arguments, &'static str> {
    let owner = body
        .owner
        .ok_or("defining closure has no enclosing owner")?;
    let parameters = defining_parameters(facts, owner)?;
    bind_arguments(facts, instance, body, &parameters)
}

pub(crate) fn arguments(
    facts: &Declarations,
    instance: Instance,
    body: Body,
) -> Result<Arguments, &'static str> {
    let owner = body.owner.ok_or("generic closure has no enclosing owner")?;
    let parameters = match facts
        .lifetimes
        .callables
        .generic_parameters(ItemId(owner.0))
    {
        Some(parameters) => parameters,
        None if !facts.generics.has_parameters(owner) => &[],
        None => return Err("missing enclosing closure parameters"),
    };
    bind_arguments(facts, instance, body, parameters)
}

fn bind_arguments(
    facts: &Declarations,
    instance: Instance,
    body: Body,
    parameters: &[ParameterId],
) -> Result<Arguments, &'static str> {
    let Some(Kind::Closure { arguments, .. }) = facts.types.kind(instance.ty()) else {
        return Err("closure arguments require a canonical closure instance");
    };
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("invalid closure arguments")?
        .collect::<Vec<_>>();
    let inherited = body.inherited as usize;
    if arguments.len() != inherited.saturating_add(parameters.len()) {
        return Err("closure instance argument count differs from its enclosing template");
    }
    let captures = arguments[..inherited]
        .iter()
        .map(|argument| match argument {
            Argument::Lifetime(region) => Ok(*region),
            _ => Err("closure inherited argument is not a lifetime"),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Arguments {
        bindings: parameters
            .iter()
            .copied()
            .zip(arguments[inherited..].iter().copied())
            .collect(),
        captures,
    })
}

pub(crate) fn execution(
    facts: &mut Declarations,
    instance: Instance,
    mut execution: Execution,
) -> Result<Execution, &'static str> {
    let Some(Kind::Closure { identity, .. }) = facts.types.kind(instance.ty()) else {
        return Err("closure execution requires a canonical instance");
    };
    let source = *facts
        .closures
        .identity(identity)
        .ok_or("missing closure source")?;
    let mut inputs = arguments(facts, instance, source)?;
    if inputs.bindings.is_empty() {
        return Ok(execution);
    }
    let (mut parameters, universals) = substitution(facts, &mut inputs, execution.universals)?;
    execution.universals = universals;
    execution.self_ty = parameters.apply(&mut facts.types, execution.self_ty)?;
    execution.signature =
        parameters.signature(&mut facts.types, execution.signature, execution.universals)?;
    facts.body_work.canonical_steps += parameters.steps;
    Ok(execution)
}

/// Retain every proof slot while substituting a noncapturing body's call roots.
pub(crate) fn signature(
    facts: &mut Declarations,
    instance: Instance,
    source: Body,
) -> Result<(crate::types::TypeId, u32), &'static str> {
    let mut inputs = arguments(facts, instance, source)?;
    let (mut parameters, universals) = substitution(facts, &mut inputs, source.universals)?;
    let signature = parameters.signature(&mut facts.types, source.signature, universals)?;
    facts.body_work.canonical_steps += parameters.steps;
    Ok((signature, universals))
}

fn substitution(
    facts: &mut Declarations,
    inputs: &mut Arguments,
    universals: u32,
) -> Result<(Bound, u32), &'static str> {
    let mut regions = Bound::universals(universals);
    for (_, argument) in &mut inputs.bindings {
        *argument = regions.apply_argument(&mut facts.types, *argument)?;
    }
    let count = u32::try_from(regions.universal_count())
        .map_err(|_| "too many closure argument lifetimes")?;
    let universals = universals
        .checked_add(count)
        .ok_or("closure lifetime overflow")?;
    let parameters = Bound::parameters(&facts.types, &inputs.bindings)?;
    facts.body_work.canonical_steps += regions.steps;
    Ok((parameters, universals))
}
