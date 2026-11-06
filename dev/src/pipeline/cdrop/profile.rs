//! The first executable profile retains closed ordinary Struct Drop bodies.
use crate::body::drops::{
    glue,
    schedule::{Call, Guard},
};
use crate::{
    body::{
        ir::{Body, Owner},
        regions::Contract,
    },
    borrow::destruction::{Condition, Operation, Plan},
    sema::model::Declarations,
    types::{GenericArgs, ItemId, Kind, TypeId},
};

pub(super) fn select(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    plan: &Plan,
    recipes: &glue::Recipes,
    charge: &crate::body::copy_requirements::OriginalIndexBudget,
    work: &mut crate::body::defining_cleanups::Work,
) -> Result<Vec<Call>, &'static str> {
    if !matches!(
        body.owner,
        Owner::Function {
            execution: None,
            ..
        }
    ) || !body.const_context
        || body.instance.is_some()
        || body.source.source().is_none()
    {
        return Err("constant cleanup execution requires its actual closed source function");
    }
    if !body.temporaries.is_empty() {
        return Err("constant materialized-temporary cleanup execution is not implemented");
    }
    charge.charge(plan.events.len(), 0)?;
    if plan
        .events
        .iter()
        .any(|event| matches!(event.effect, crate::borrow::destruction::Effect::Birth(_)))
    {
        return Err("constant cleanup execution requires its finite local event profile");
    }
    charge.charge(plan.cleanups.len(), 0)?;
    let count = plan
        .cleanups
        .iter()
        .filter(|cleanup| cleanup.node.is_some())
        .count();
    charge.charge(plan.cleanups.len(), count.saturating_mul(size_of::<Call>()))?;
    let mut selected = Vec::with_capacity(count);
    for (index, cleanup) in plan.cleanups.iter().enumerate() {
        let Some(id) = cleanup.node else {
            continue;
        };
        let node = plan
            .nodes
            .get(id.0 as usize)
            .ok_or("missing executable cleanup node")?;
        let Operation::Glue(id) = node.operation else {
            return Err(
                "constant recursive field/array/variant cleanup execution is not implemented",
            );
        };
        let guard = match node.condition {
            Condition::Always => Guard::ALWAYS,
            Condition::Initialized(flag) => {
                charge.charge(
                    plan.flags
                        .len()
                        .checked_ilog2()
                        .map_or(1, |n| n as usize + 1),
                    0,
                )?;
                Guard::initialized(
                    plan.flags
                        .binary_search(&flag)
                        .map_err(|_| "cleanup guard lost its actual plan flag")?,
                )?
            }
        };
        let target = body
            .place_type(cleanup.candidate.place)
            .ok_or("executable cleanup lost its exact place type")?;
        if cleanup.live_type != Some(target) {
            return Err("executable cleanup changed its complete initialized subject");
        }
        let recipe = *recipes.get(id).ok_or("missing executable cleanup recipe")?;
        if !matches!(cleanup.candidate.place, crate::body::ir::Place::Local(_)) {
            return Err("constant nonlocal cleanup execution is not implemented");
        }
        let instance = method(facts, contract, target, recipe, charge, work)?;
        selected.push(Call {
            cleanup: u32::try_from(index).map_err(|_| "cleanup ordinal overflow")?,
            place: cleanup.candidate.place,
            target,
            instance,
            recipe: id,
            guard,
        });
    }
    Ok(selected)
}

fn method(
    facts: &mut Declarations,
    contract: &Contract,
    target: TypeId,
    recipe: glue::Recipe,
    charge: &crate::body::copy_requirements::OriginalIndexBudget,
    work: &mut crate::body::defining_cleanups::Work,
) -> Result<crate::types::instance::Instance, &'static str> {
    charge.charge(12, 0)?;
    let Some(Kind::Adt(item, GenericArgs::EMPTY)) = facts.types.kind(target) else {
        return Err("constant generic/structural cleanup execution is not implemented");
    };
    let definition = facts
        .aggregates
        .get(item)
        .ok_or("cleanup nominal is not complete")?;
    if definition.kind != crate::types::aggregate::Kind::Struct
        || definition.repr.packed.is_some()
        || facts.types.contains_parameters(target) != Some(false)
        || recipe.ty != target
    {
        return Err("constant cleanup requires its actual closed ordinary Struct shape");
    }
    if !matches!(recipe.kind, glue::Shape::Fields(range) if range.start == range.end) {
        return Err("constant recursive field cleanup execution is not implemented");
    }
    let Some(glue::Method::Definition(method) | glue::Method::Unresolved(method)) = recipe.method
    else {
        return Err("constant cleanup requires its actual written Drop method");
    };
    let state = contract
        .builtins
        .as_deref()
        .and_then(|b| b.defining())
        .ok_or("cleanup has no actual completed defining State")?;
    state.require_execution_target(&facts.types, target, work)?;
    let instance = crate::sema::destructors::select(facts, target)?
        .ok_or("cleanup lost its actual selected Drop application")?;
    crate::types::destruction::validate_method(
        &mut facts.types,
        instance,
        ItemId(method.0),
        target,
    )?;
    if !facts.is_const_function(method)
        || facts.const_default_member(instance)?.is_some()
        || facts.recorded_caller_tracking(method)?
    {
        return Err("constant cleanup requires its genuine untracked written const Drop body");
    }
    facts.check_const_member_execution(method)?;
    facts
        .source_bindings
        .as_deref()
        .ok_or("cleanup lost its actual source catalog")?
        .check_const_call(method)?;
    Ok(instance)
}
