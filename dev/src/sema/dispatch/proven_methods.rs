//! Caller data borrows a matching completed selection without performing another search.
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::{header_attributes, model::Declarations},
    types::{Kind, TypeId, instance::Instance, parameters::predicates::Predicate},
};
#[cfg(test)]
#[path = "../../../tests/invariants/relationship_ports.rs"]
mod tests;

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
        let (fold, _) = facts.selected_implementation_substitution(
            implementation,
            arguments,
            &|steps, bytes| payments.charge(steps, bytes),
        )?;
        if !matches_predicate(&mut facts.types, fold, predicate, goal, &|s, b| {
            payments.charge(s, b)
        })? {
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

fn matches_goal(
    facts: &mut Declarations,
    implementation: DefId,
    arguments: crate::types::GenericArgs,
    predicate: Predicate,
    goal: Predicate,
    pay: crate::support::retained::Payment<'_>,
) -> Result<bool, &'static str> {
    let (fold, _) =
        facts.selected_implementation_substitution(implementation, arguments, &|s, b| pay(s, b))?;
    matches_predicate(&mut facts.types, fold, predicate, goal, pay)
}
fn matches_predicate(
    types: &mut crate::types::Store,
    mut fold: crate::types::substitute::Bound,
    predicate: Predicate,
    goal: Predicate,
    pay: crate::support::retained::Payment<'_>,
) -> Result<bool, &'static str> {
    let actual = fold.predicate_charged(types, predicate, &|s, b| pay(s, b))?;
    // Permission remains in the retained proof. ApplicationId includes every
    // trait argument; matching its trait name alone would be insufficient.
    Ok(actual.ty == goal.ty
        && actual.bound.trait_parts().map(|(id, _)| id)
            == goal.bound.trait_parts().map(|(id, _)| id))
}

/// Structural data from an already completed site selection, never a new search
/// or call permission. The complete target includes its arguments and signature.
pub(crate) fn selected_callable(
    facts: &mut Declarations,
    source: NodeKey,
    callable: TypeId,
    pay: crate::support::retained::Payment<'_>,
) -> Result<Option<TypeId>, &'static str> {
    pay(1, 0)?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(callable) else {
        return Ok(None);
    };
    let declaration = DefId(item.0);
    if facts.associated.trait_owner(declaration).is_none() {
        return Ok(None);
    }
    let count = facts
        .generics
        .trait_selections
        .get(&source)
        .map_or(0, Vec::len);
    if count == 0 {
        return Ok(None);
    }
    let goal = callable_requirement(facts, callable, pay)?
        .ok_or("selected method has no actual trait goal")?;
    let arguments = super::arguments::Arguments::new_paid(facts, callable, pay)?;
    let mut result = None;
    for index in 0..count {
        pay(1, 0)?;
        let selection = &facts.generics.trait_selections[&source][index];
        let (owner, actual) = (selection.owner, selection.arguments);
        let registry = facts
            .generics
            .implementations
            .as_deref()
            .ok_or("selected callable registry is absent")?;
        if !registry.is_validated(owner) {
            return Err("selected callable implementation is not validated");
        }
        let header = registry.checked_entry(registry.entry_key(owner)?)?;
        pay(header.methods.len().max(1).ilog2() as usize + 1, 0)?;
        if header
            .methods
            .binary_search_by_key(&declaration.0, |(member, _)| member.0)
            .is_err()
        {
            continue;
        }
        let predicate = header.predicate;
        if !matches_goal(facts, owner, actual, predicate, goal, pay)? {
            continue;
        }
        let selected =
            crate::sema::specialization::selected_method(facts, owner, actual, declaration)?;
        selected.current(facts)?;
        let target = if selected.definition() == declaration {
            callable
        } else {
            let (owner, actual) = selected
                .implementation()
                .ok_or("selected callable lost its defining implementation")?;
            arguments.target_paid(facts, owner, actual, selected.definition(), pay)?
        };
        if facts.types.contains_projections(target) != Some(false) {
            return Err(
                "selected source callable projections require their actual dispatch normalization contract",
            );
        }
        if result.is_some_and(|previous| previous != target) {
            return Err("selected callable has conflicting complete targets");
        }
        result = Some(target);
    }
    Ok(result)
}

/// Substitute the genuine callable formals in its trait or validated impl goal.
/// This supplies no evidence and does not choose an implementation.
pub(crate) fn callable_requirement(
    facts: &mut Declarations,
    callable: TypeId,
    pay: crate::support::retained::Payment<'_>,
) -> Result<Option<Predicate>, &'static str> {
    pay(1, 0)?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(callable) else {
        return Err("callable requirement has no actual function item");
    };
    let definition = DefId(item.0);
    let predicate = if let Some(owner) = facts.associated.trait_owner(definition) {
        *facts
            .generics
            .trait_self_bounds
            .get(&owner)
            .ok_or("callable requirement has no trait Self predicate")?
    } else if let Some(owner) = facts
        .associated
        .owner(definition)
        .and_then(|owner| facts.associated.get(owner))
    {
        let registry = facts
            .generics
            .implementations
            .as_deref()
            .ok_or("callable requirement has no implementation registry")?;
        if !registry.is_validated(owner.self_definition) {
            return Err("callable requirement implementation is not validated");
        }
        let header = registry.checked_entry(registry.entry_key(owner.self_definition)?)?;
        if header.source != owner.source {
            return Err("callable requirement changed its actual implementation source");
        }
        header.predicate
    } else {
        return Ok(None);
    };
    let arguments = super::arguments::Arguments::new_paid(facts, callable, pay)?;
    let mut fold = arguments.substitution_paid(facts, pay)?;
    fold.predicate_charged(&mut facts.types, predicate, &|s, b| pay(s, b))
        .map(Some)
}

/// Reconstruct structural request data for a fresh dispatch proof. Neither the
/// selected item nor its recorded goal supplies implementation permission.
pub(crate) fn requested_callable(
    facts: &mut Declarations,
    member: DefId,
    selected: TypeId,
    recorded: Option<Predicate>,
    pay: crate::support::retained::Payment<'_>,
) -> Result<TypeId, &'static str> {
    let goal = callable_requirement(facts, selected, pay)?
        .ok_or("selected callable has no actual implementation goal")?;
    if recorded.is_some_and(|recorded| !same_goal(recorded, goal)) {
        return Err("selected callable changed its complete recorded method goal");
    }
    let arguments = super::arguments::Arguments::new_paid(facts, selected, pay)?;
    let requested = arguments.requested_paid(facts, member, goal, pay)?;
    let actual = callable_requirement(facts, requested, pay)?
        .ok_or("requested callable lost its actual trait goal")?;
    let signature = |ty| match facts.types.kind(ty) {
        Some(Kind::FunctionItem { signature, .. }) => Some(signature),
        _ => None,
    };
    if !same_goal(actual, goal) || signature(requested) != signature(selected) {
        return Err("requested callable changed its complete goal or applied signature");
    }
    Ok(requested)
}
fn same_goal(a: Predicate, b: Predicate) -> bool {
    a.ty == b.ty && a.bound.trait_parts().map(|p| p.0) == b.bound.trait_parts().map(|p| p.0)
}
