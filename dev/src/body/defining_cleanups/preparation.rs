//! Declaration collection records required owners; finalization binds the actual body.
use super::{
    Arc, Body, BodyOrigin, Contract, Declarations, DefId, Definition, Owner, Prepared, State, Work,
    claims, definition, validate_contract,
};
use crate::body::regions::BuiltinRequirements;

pub(crate) fn mark_required(
    facts: &Declarations,
    function: &crate::sema::model::Function,
    contract: &mut Contract,
) -> Result<(), &'static str> {
    if function.body.map(BodyOrigin::Source) != Some(contract.source) {
        return Err("defining cleanup source function changed its actual body");
    }
    let Some(required) = definition(facts, function.definition, contract.source, &mut 0)? else {
        return Ok(());
    };
    validate_contract(required, contract)?;
    let builtins = contract
        .builtins
        .get_or_insert_with(|| Arc::new(BuiltinRequirements::empty()));
    Arc::make_mut(builtins).set_defining(Some(Arc::new(State::Required(required))));
    Ok(())
}

/// Reconstruct an unpaid request from the actual current imported declaration.
pub(crate) fn mark_imported_required(
    facts: &Declarations,
    definition: DefId,
    contract: &mut Contract,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(5);
    if contract.source.source().is_some() {
        return Err("imported defining cleanup requires its actual imported body");
    }
    let Some(required) = super::definition(facts, definition, contract.source, steps)? else {
        return Ok(());
    };
    validate_contract(required, contract)?;
    let builtins = contract
        .builtins
        .as_mut()
        .and_then(Arc::get_mut)
        .ok_or("imported defining cleanup requires its fresh owned request carrier")?;
    if builtins.defining().is_some() {
        return Err("imported defining cleanup cannot reuse a prior request or proof");
    }
    builtins.set_defining(Some(Arc::new(State::Required(required))));
    Ok(())
}

pub(crate) fn validate_imported_required(
    facts: &Declarations,
    definition: DefId,
    contract: &Contract,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(3);
    let expected = super::definition(facts, definition, contract.source, steps)?;
    let actual = contract.builtins.as_deref().and_then(|b| b.defining());
    match (expected, actual) {
        (None, None) => Ok(()),
        (Some(expected), Some(State::Required(actual))) if expected == *actual => Ok(()),
        _ => Err("source original changed its unpaid defining cleanup request"),
    }
}

pub(crate) fn prepare(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
) -> Result<(), String> {
    let mut work = Work::new(1_000_000);
    let result = counted(facts, body, contract, &mut work);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work.steps);
    result.map_err(String::from)
}
pub(crate) fn begin(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
) -> Result<(), String> {
    let mut work = Work::new(1_000_000);
    let result = begin_counted(facts, body, contract, &mut work);
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work.steps);
    result.map_err(String::from)
}
fn begin_counted(
    facts: &Declarations,
    body: &Body,
    contract: &mut Contract,
    work: &mut Work,
) -> Result<(), &'static str> {
    if body.instance.is_some() {
        return counted(facts, body, contract, work);
    }
    let Owner::Function {
        definition: id,
        signature,
        execution,
    } = body.owner
    else {
        if contract.requires_defining_cleanup_preparation() {
            work.charge(1)?;
            return Err("defining cleanup owner is not a function");
        }
        return Ok(());
    };
    if let Some(builtins) = contract.builtins.as_mut() {
        let stale = match builtins.defining() {
            Some(State::Prepared(prepared))
                if prepared.definition.function == id
                    && prepared.definition.source == body.source =>
            {
                Some(prepared.definition)
            }
            _ => None,
        };
        if let Some(required) = stale {
            work.charge(1)?;
            Arc::make_mut(builtins).set_defining(Some(Arc::new(State::Required(required))));
        }
    }
    let Some(actual) = counted_definition(facts, id, body.source, work)? else {
        if contract.requires_defining_cleanup_preparation() {
            work.charge(1)?;
            return Err("defining cleanup required carrier has another owner");
        }
        return Ok(());
    };
    validate_contract(actual, contract)?;
    if execution.is_some() || signature != actual.signature || contract.signature != Some(signature)
    {
        return Err("defining cleanup original changed its actual signature");
    }
    let builtins = contract
        .builtins
        .as_mut()
        .ok_or("defining cleanup original has no required owner carrier")?;
    let state = builtins
        .defining()
        .ok_or("defining cleanup original lost its required owner carrier")?;
    let required = match state {
        State::Required(required) => *required,
        State::Prepared(prepared) => prepared.definition,
    };
    if required != actual {
        return Err("defining cleanup carrier changed its actual member or trait owner");
    }
    Ok(())
}
fn counted(
    facts: &Declarations,
    body: &Body,
    contract: &mut Contract,
    work: &mut Work,
) -> Result<(), &'static str> {
    if body.instance.is_some() {
        return clear_instance(facts, body, contract, work);
    }
    let owner = match body.owner {
        Owner::Function {
            definition: id,
            signature,
            execution,
        } => counted_definition(facts, id, body.source, work)?
            .map(|actual| (id, signature, execution, actual)),
        _ => None,
    };
    let Some((id, signature, execution, actual)) = owner else {
        if contract
            .builtins
            .as_deref()
            .is_some_and(BuiltinRequirements::has_defining)
        {
            work.charge(1)?;
            return Err("defining cleanup carrier belongs to another actual function owner");
        }
        return Ok(());
    };
    validate_contract(actual, contract)?;
    if execution.is_some() || signature != actual.signature || contract.signature != Some(signature)
    {
        return Err("defining cleanup requires the exact original callable signature");
    }
    let builtins = contract
        .builtins
        .as_ref()
        .ok_or("defining cleanup original has no required owner carrier")?;
    let state = builtins
        .defining()
        .ok_or("defining cleanup original lost its required owner carrier")?;
    let required = match state {
        State::Required(required) => *required,
        State::Prepared(prepared) => prepared.definition,
    };
    if required != actual {
        return Err("defining cleanup required carrier belongs to another declaration");
    }
    work.charge(1)?;
    let defining_environment = facts
        .generics
        .body_environment(id)
        .ok_or("defining cleanup original has no actual defining environment")?;
    let environment = if let Some(environment) = &builtins.environment {
        if !Arc::ptr_eq(environment, defining_environment) {
            return Err("defining cleanup changed its actual defining environment");
        }
        Arc::clone(environment)
    } else {
        Arc::clone(defining_environment)
    };
    let role = claims::Role::current(facts, crate::support::bindings::LangItem::Destruct, work)?;
    let copy_role = claims::Role::current(facts, crate::support::bindings::LangItem::Copy, work)?;
    let claims = claims::Claims::new(facts, &environment, role, copy_role, work)?;
    let prepared = Prepared {
        definition: actual,
        owned_signature: signature,
        const_context: body.const_context,
        environment: Arc::clone(&environment),
        role,
        copy_role,
        claims,
    };
    let builtins = Arc::make_mut(
        contract
            .builtins
            .as_mut()
            .ok_or("defining cleanup carrier disappeared")?,
    );
    builtins.environment = Some(environment);
    builtins.set_defining(Some(Arc::new(State::Prepared(prepared))));
    Ok(())
}

fn counted_definition(
    facts: &Declarations,
    id: DefId,
    source: BodyOrigin,
    work: &mut Work,
) -> Result<Option<Definition>, &'static str> {
    if facts
        .associated
        .trait_owner(id)
        .is_some_and(|owner| facts.associated.constant_trait(owner) == Some(true))
    {
        // Include the completed declaration-kind and member-constness choice
        // lookups, before any attempt or malformed/pending result.
        work.charge(32)?;
    } else if super::generic_const_function(facts, id) {
        work.charge(16)?;
    }
    let result = definition(facts, id, source, &mut work.steps);
    let charged = work.charge(0);
    result.and_then(|definition| charged.map(|()| definition))
}

fn clear_instance(
    facts: &Declarations,
    body: &Body,
    contract: &mut Contract,
    work: &mut Work,
) -> Result<(), &'static str> {
    if let Some(builtins) = contract.builtins.as_mut().filter(|r| r.has_defining()) {
        work.charge(27)?;
        let state = builtins
            .defining()
            .ok_or("missing defining cleanup instance carrier")?;
        let required = match state {
            State::Required(required) => *required,
            State::Prepared(prepared) => prepared.definition,
        };
        let Owner::Function {
            definition,
            signature,
            ..
        } = body.owner
        else {
            return Err("defining cleanup instance has no actual function owner");
        };
        let instance = body.instance.ok_or("missing defining cleanup instance")?;
        let header = facts
            .function_header(definition)?
            .ok_or("defining cleanup instance header is pending")?;
        if required.function != definition
            || required.source != body.source
            || counted_definition(facts, definition, body.source, work)? != Some(required)
            || header.body != Some(required.source)
            || header.signature != required.signature
            || !matches!(facts.types.kind(instance.ty()), Some(crate::types::Kind::FunctionItem { item, signature: actual, .. })
                if item.0 == definition.0 && actual == signature)
        {
            return Err("defining cleanup specialization changed its actual member or callable");
        }
        Arc::make_mut(builtins).set_defining(None);
    }
    Ok(())
}
