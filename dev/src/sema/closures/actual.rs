//! Selected closure arguments retain the original source binder before substitution.
use super::{Body, Execution};
use crate::{
    sema::model::Declarations,
    types::{
        ItemId, Kind, RegionId, arguments::Argument, instance::Instance, parameters::ParameterId,
        substitute::Bound,
    },
};
pub(crate) struct Arguments {
    pub bindings: Vec<(ParameterId, Argument)>,
    pub captures: Vec<RegionId>,
}
pub(crate) fn arguments(
    facts: &Declarations,
    instance: Instance,
    body: Body,
) -> Result<Arguments, &'static str> {
    let Some(Kind::Closure { arguments, .. }) = facts.types.kind(instance.ty()) else {
        return Err("closure arguments require a canonical closure instance");
    };
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
    let mut regions = Bound::universals(execution.universals);
    for (_, argument) in &mut inputs.bindings {
        *argument = regions.apply_argument(&mut facts.types, *argument)?;
    }
    let count = u32::try_from(regions.universal_count())
        .map_err(|_| "too many closure argument lifetimes")?;
    execution.universals = execution
        .universals
        .checked_add(count)
        .ok_or("closure lifetime overflow")?;
    let mut parameters = Bound::parameters(&facts.types, &inputs.bindings)?;
    execution.self_ty = parameters.apply(&mut facts.types, execution.self_ty)?;
    execution.signature =
        parameters.signature(&mut facts.types, execution.signature, execution.universals)?;
    facts.body_work.canonical_steps += regions.steps + parameters.steps;
    Ok(execution)
}
