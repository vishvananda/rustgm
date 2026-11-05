//! Deferred originals and generated entry ownership are distinct phase identities.
use crate::{
    body::ir::{Body, Owner},
    resolve::model::DefId,
    sema::model::Declarations,
};

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
