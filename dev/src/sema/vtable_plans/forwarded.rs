//! A reference to a callable object forwards through the referenced vtable.
use super::{Operator, Slot, operators, shapes};
use crate::{
    resolve::expansion::Report,
    sema::{model::Declarations, traits::object_bounds},
    types::{
        GenericArgs, ItemId, Kind, TypeId, arguments::Argument, callable::Convention,
        parameters::predicates::Bound,
    },
};

pub(super) fn target(
    report: &Report,
    facts: &mut Declarations,
    concrete: TypeId,
    slot: Slot,
) -> Result<Operator, String> {
    let requested = slot.method.require_callable()?;
    // The erased receiver points to the concrete storage. Even a consuming
    // call through it must follow this extra pointer to a thin reference.
    let reference = facts.types.storage_head(concrete)?;
    let (object, kind, references) =
        operators::reference_subject(&facts.types, reference, requested, 1)?;
    let shape = shapes::prepare(report, facts, object)?;
    let object = facts
        .objects
        .shape(shape)
        .ok_or("missing forwarded callable shape")?
        .object;
    let arguments = facts
        .types
        .trait_application(slot.parent)
        .ok_or("missing forwarded callable application")?
        .arguments;
    let call = operators::virtual_slot(facts, shape, kind, arguments)?;
    let result = result(facts, object, arguments)?;
    let execution = signature(facts, concrete, arguments, result)?;
    facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing forwarded callable plans")?
        .charge(references as usize)?;
    Ok(Operator {
        call,
        execution,
        dereferences: references.saturating_sub(2),
    })
}

fn result(
    facts: &mut Declarations,
    object: TypeId,
    arguments: GenericArgs,
) -> Result<TypeId, &'static str> {
    let output = facts
        .associated
        .callable_output()
        .ok_or("missing callable Output declaration")?;
    let (bounds, steps) = object_bounds::get(facts, object)?;
    let equations = bounds
        .equalities
        .get(&ItemId(output.0))
        .ok_or("missing forwarded callable output equation")?;
    let storage = facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing forwarded callable storage")?;
    let before = storage.erase.steps;
    let mut selected = None;
    for &equation in equations.as_ref() {
        let predicate = object_bounds::graph::quantified_value(&facts.types, equation)?;
        let (Some(Kind::Projection(projection)), Bound::ProjectionEqual(result)) =
            (facts.types.kind(predicate.ty), predicate.bound)
        else {
            return Err("invalid forwarded callable output equation");
        };
        let application = facts
            .types
            .trait_application(projection.trait_application)
            .ok_or("missing forwarded output application")?;
        if storage
            .erase
            .apply_arguments(&mut facts.types, application.arguments)?
            == arguments
        {
            let result = storage.erase.apply(&mut facts.types, result)?;
            if selected
                .replace(result)
                .is_some_and(|prior| prior != result)
            {
                return Err("forwarded callable has conflicting outputs");
            }
        }
    }
    storage.charge(
        steps
            .saturating_add(equations.len())
            .saturating_add(storage.erase.steps.saturating_sub(before)),
    )?;
    selected.ok_or("missing output for the forwarded callable application")
}

fn signature(
    facts: &mut Declarations,
    concrete: TypeId,
    arguments: GenericArgs,
    result: TypeId,
) -> Result<TypeId, &'static str> {
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("missing forwarded callable arguments")?;
    let (Some(Argument::Type(tuple)), None) = (arguments.next(), arguments.next()) else {
        return Err("forwarded callable requires its complete tuple argument");
    };
    let Some(Kind::Tuple(inputs)) = facts.types.kind(tuple) else {
        return Err("forwarded callable argument is not a tuple");
    };
    let receiver = facts.types.intern(Kind::Pointer {
        mutable: false,
        pointee: concrete,
    })?;
    let inputs = facts
        .types
        .list(inputs)
        .ok_or("missing forwarded callable inputs")?;
    let mut parameters = Vec::with_capacity(inputs.len().saturating_add(1));
    parameters.push(receiver);
    parameters.extend_from_slice(inputs);
    facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing forwarded callable storage")?
        .charge(parameters.len())?;
    let parameters = facts.types.intern_list(&parameters)?;
    facts.types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    })
}
