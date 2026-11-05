//! Deferred originals and generated entry ownership are distinct phase identities.
mod creation;
mod disposal;
mod framing;
mod permit;
#[cfg(test)]
#[path = "../../tests/invariants/statepermit.rs"]
mod tests;
use crate::{
    body::ir::{Body, Owner},
    resolve::model::DefId,
    sema::model::Declarations,
};
pub(crate) use creation::{Constructed, constructor};
pub(crate) use disposal::build as drop_entry;
pub(crate) use framing::{field, transfer};
pub(crate) use permit::Permit;

/// Ordinary proof cannot publish an entry whose generated factory is absent.
pub(crate) fn require_ordinary_body(body: &Body) -> Result<(), &'static str> {
    if matches!(body.owner, Owner::FutureEntry { .. }) {
        return Err("future entry requires its actual checked generated factory proof");
    }
    Ok(())
}

/// A declaration's deferred original cannot service its constructor demand.
pub(crate) fn require_ordinary_original(
    facts: &Declarations,
    definition: DefId,
) -> Result<(), &'static str> {
    if facts.opaques.owners.contains_key(&definition)
        && facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .map(|function| facts.authored_async_body(function))
            .transpose()?
            .flatten()
            .is_some()
    {
        return Err("async constructor requires its checked generated future family");
    }
    Ok(())
}

/// Authenticate the actual source body; this supplies no constructor or native permit.
pub(crate) fn original(facts: &Declarations, body: &Body) -> Result<DefId, &'static str> {
    let Owner::AsyncBody {
        identity,
        future,
        signature,
    } = body.owner
    else {
        return Err("deferred original requires its actual AsyncBody owner");
    };
    let definition = facts
        .opaques
        .declarations
        .get(&identity)
        .ok_or("deferred original lost its actual opaque owner")?
        .owner;
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or("deferred original lost its actual function ordinal")?;
    let expected = facts
        .authored_async_body(function)?
        .ok_or("deferred original lacks its actual authored input contract")?;
    if expected.identity != identity
        || expected.future != future
        || expected.signature != signature
        || body.result != expected.output
        || body.instance.is_some()
        || body.const_context
        || function.body.map(Into::into) != Some(body.source)
    {
        return Err("deferred original changed its owner, capture frame, Output or body");
    }
    Ok(definition)
}

/// Borrowed original proof inputs share the actual source body's signature/frame.
pub(crate) fn original_contract(
    facts: &Declarations,
    body: &Body,
    contract: &super::regions::Contract,
) -> Result<DefId, &'static str> {
    let definition = original(facts, body)?;
    let header = facts
        .function_header(definition)?
        .ok_or("deferred original lost its actual callable contract")?;
    let expected = body.source.source().and_then(|source| {
        facts
            .nested_outputs
            .as_deref()
            .and_then(|rows| rows.get(&source))
    });
    let same_output = match (&contract.output, expected) {
        (Some(actual), Some(expected)) => std::sync::Arc::ptr_eq(actual, expected),
        (None, None) => true,
        _ => false,
    };
    if contract.key != body.key()
        || contract.source != body.source
        || contract.signature != body.owner.signatures().map(|(_, signature)| signature)
        || contract.universals != header.universals
        || !same_output
    {
        return Err("deferred original changed its actual signature, binder or output link");
    }
    Ok(definition)
}

pub(crate) fn original_template(
    facts: &Declarations,
    template: &super::templates::Template,
    contract: &super::regions::Contract,
) -> Result<DefId, &'static str> {
    let definition = original_contract(facts, &template.body, contract)?;
    if let Some(environment) = facts
        .function_header(definition)?
        .and_then(|header| header.environment)
        && !std::sync::Arc::ptr_eq(environment, &template.environment)
    {
        return Err("deferred original changed its actual declaration environment");
    }
    Ok(definition)
}
