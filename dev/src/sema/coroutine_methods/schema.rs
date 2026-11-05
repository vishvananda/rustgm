//! Demand authenticates the canonical contract; unused alternatives remain ordinary declarations.
use crate::{
    resolve::model::DefId,
    sema::{associated::OwnerKind, model::Declarations},
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, TypeId,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

pub(super) fn require(facts: &Declarations) -> Result<(DefId, DefId), &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("future evidence requires its actual source catalog")?;
    let future = catalog
        .validated_language(LangItem::Future)?
        .ok_or("future evidence requires its completed actual trait")?;
    let output = catalog
        .validated_language(LangItem::FutureOutput)?
        .ok_or("future evidence requires its completed actual Output")?;
    let owner_id = facts
        .associated
        .owner(future.definition)
        .ok_or("future evidence lost its actual trait owner")?;
    let owner = facts
        .associated
        .get(owner_id)
        .ok_or("future evidence lost its actual trait owner")?;
    let header = facts
        .generics
        .traits
        .as_deref()
        .and_then(|traits| traits.header(ItemId(future.definition.0)))
        .ok_or("future evidence lost its canonical trait header")?;
    let inputs = header.inputs.source_ref()?;
    if !matches!(owner.kind, OwnerKind::Trait { definition, auto_:false, unsafe_:false, constant:false } if definition == future.definition)
        || owner.source != Some(future.origin)
        || facts
            .associated
            .schema(future.definition)
            .is_none_or(|schema| schema.self_type != header.self_type || schema.captures != 0)
        || !header.parameters.is_empty()
        || !header.regions.is_empty()
        || output.parent != Some(future.definition)
        || facts.associated.trait_owner(output.definition) != Some(future.definition)
    {
        return Err("demanded Future differs from its actual canonical trait contract");
    }
    let members = facts
        .associated
        .owner_members(owner_id)
        .ok_or("future evidence lost its completed actual member inventory")?;
    facts.charge_caller_inputs(
        inputs
            .predicates
            .len()
            .saturating_mul(4)
            .saturating_add(members.len())
            .saturating_add(header.results.len()),
        0,
    )?;
    for input in header.inputs.view().iter() {
        if !sized_clause(facts, input.value, header.self_type, true)? {
            return Err(
                "demanded Future supertrait promises require additional language implementation proof",
            );
        }
    }
    for &member in members {
        if member == output.definition {
            continue;
        }
        let poll = catalog
            .validated_language(LangItem::FuturePoll)?
            .ok_or("demanded Future has an unsupported actual member")?;
        if poll.definition != member
            || poll.parent != Some(future.definition)
            || facts.associated.trait_owner(member) != Some(future.definition)
            || facts
                .function_header(member)?
                .is_none_or(|header| header.declaration.source() != Some(poll.origin))
        {
            return Err(
                "demanded Future member differs from its actual completed polling declaration",
            );
        }
    }
    let projection = output_contract(facts, output.definition)?;
    for &(item, input) in &header.results {
        if item != ItemId(output.definition.0)
            || !sized_clause(facts, input.value, projection, false)?
        {
            return Err("demanded Future has unsupported actual associated-result promises");
        }
    }
    Ok((future.definition, output.definition))
}

fn output_contract(facts: &Declarations, member: DefId) -> Result<TypeId, &'static str> {
    let contract = facts.associated.require_projection_header(member)?;
    let schema = facts
        .types
        .projection_schema(ItemId(member.0))
        .ok_or("future Output lost its actual projection schema")?;
    if facts
        .generics
        .parameters
        .get(&member)
        .is_some_and(|parameters| !parameters.is_empty())
        || facts
            .associated
            .families
            .get(&member)
            .is_some_and(|family| {
                family.total != 0 || family.captures != 0 || family.requirements.is_none()
            })
    {
        return Err("demanded Future Output requires its completed zero-own-argument family");
    }
    facts.charge_caller_inputs(contract.bounds.len().saturating_mul(4), 0)?;
    let mut sized = false;
    for &clause in &contract.bounds {
        if !sized_clause(facts, clause, contract.projection, false)? {
            return Err(
                "demanded Future Output promises require additional result implementation proof",
            );
        }
        sized = true;
    }
    if schema.sized && !sized {
        return Err("future Output Sized promise lacks its actual canonical bound");
    }
    Ok(contract.projection)
}

fn sized_clause(
    facts: &Declarations,
    clause: Clause,
    subject: TypeId,
    meta: bool,
) -> Result<bool, &'static str> {
    let Clause::Predicate(predicate) = clause else {
        return Ok(false);
    };
    if predicate.ty != subject {
        return Ok(false);
    }
    if predicate.bound == Bound::Sized {
        return Ok(true);
    }
    let Bound::Trait(id) = predicate.bound else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("future declaration has an invalid sizing input")?;
    if application.arguments != GenericArgs::EMPTY {
        return Ok(false);
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("future declaration lost its actual source catalog")?;
    for role in [LangItem::Sized, LangItem::MetaSized, LangItem::PointeeSized] {
        if !meta && role != LangItem::Sized {
            continue;
        }
        if catalog
            .header_role(role)?
            .is_some_and(|role| role.definition.0 == application.item.0)
        {
            return Ok(true);
        }
    }
    Ok(false)
}
