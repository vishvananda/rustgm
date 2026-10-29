//! Caller data borrows a matching completed selection without performing another search.
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::{header_attributes, model::Declarations},
    types::{Kind, TypeId, instance::Instance, parameters::predicates::Predicate},
};

fn callable_goal(
    facts: &mut Declarations,
    callable: TypeId,
    owner: DefId,
) -> Result<Predicate, &'static str> {
    let predicate = *facts
        .generics
        .trait_self_bounds
        .get(&owner)
        .ok_or("caller selection lacks its declared trait predicate")?;
    let instance = Instance::new(&facts.types, callable)?;
    let Some(Kind::FunctionItem { arguments, .. }) = facts.types.kind(callable) else {
        return Err("caller selection lost its callable");
    };
    let count = facts
        .types
        .arguments(arguments)
        .ok_or("caller selection lost callable arguments")?
        .len();
    let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|_| "caller selection lacks its source budget")?;
    payments.charge(
        count.saturating_add(1),
        count.saturating_mul(
            4 * size_of::<(
                crate::types::parameters::ParameterId,
                crate::types::arguments::Argument,
            )>(),
        ),
    )?;
    let mut fold = super::callable_substitution(facts, instance)?;
    fold.predicate_charged(&mut facts.types, predicate, &|steps, bytes| {
        payments.charge(steps, bytes)
    })
}

/// This is header identity only; the original call, API and lifetime proofs remain required.
pub(in crate::sema) fn selected_definition(
    facts: &mut Declarations,
    source: NodeKey,
    callable: TypeId,
) -> Result<Option<DefId>, &'static str> {
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(callable) else {
        return Ok(None);
    };
    let declaration = DefId(item.0);
    let Some(owner) = facts.associated.trait_owner(declaration) else {
        return Ok(None);
    };
    let count = facts
        .generics
        .trait_selections
        .get(&source)
        .map_or(0, Vec::len);
    if count == 0 {
        return Ok(None);
    }
    let goal = callable_goal(facts, callable, owner)?;
    let mut result = None;
    for index in 0..count {
        facts.charge_caller_inputs(1, 0)?;
        let selection = &facts.generics.trait_selections[&source][index];
        let (implementation, arguments) = (selection.owner, selection.arguments);
        let registry = facts
            .generics
            .implementations
            .as_deref()
            .ok_or("caller selection lost its implementation registry")?;
        let header = registry
            .header(implementation)
            .ok_or("caller selection lost its implementation header")?;
        facts.charge_caller_inputs(header.methods.len().max(1).ilog2() as usize + 1, 0)?;
        if header
            .methods
            .binary_search_by_key(&declaration.0, |(member, _)| member.0)
            .is_err()
        {
            continue;
        }
        let predicate = header.predicate;
        let payments = header_attributes::source_payments(facts.must_use_contracts.as_deref())
            .map_err(|_| "caller selection lacks its source budget")?;
        let (mut fold, _) = facts.selected_implementation_substitution(
            implementation,
            arguments,
            &|steps, bytes| payments.charge(steps, bytes),
        )?;
        let actual = fold.predicate_charged(&mut facts.types, predicate, &|steps, bytes| {
            payments.charge(steps, bytes)
        })?;
        // Effect permission belongs to the retained selection proof. Only its
        // subject and complete trait application identify this callable's target.
        if actual.ty != goal.ty
            || actual.bound.trait_parts().map(|(id, _)| id)
                != goal.bound.trait_parts().map(|(id, _)| id)
        {
            continue;
        }
        let selected = crate::sema::specialization::selected_method(
            facts,
            implementation,
            arguments,
            declaration,
        )?
        .definition();
        if result.is_some_and(|previous| previous != selected) {
            return Err("caller selection has conflicting actual method targets");
        }
        result = Some(selected);
    }
    Ok(result)
}
