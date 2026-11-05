//! Authored constructor identity comes from its real typed source function/opaque owner.
use crate::{
    resolve::model::DefId,
    sema::model::Declarations,
    types::{Kind, instance::Instance},
};

/// Source identity data only: checked originals, formation and generated bodies stay separate.
pub(crate) fn constructor_definition(
    facts: &Declarations,
    instance: Instance,
) -> Result<Option<DefId>, &'static str> {
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
        return Ok(None);
    };
    let Some(function) = facts
        .function_indices
        .get(&DefId(item.0))
        .and_then(|&index| facts.functions.get(index))
    else {
        return Ok(None);
    };
    Ok(facts
        .authored_async_body(function)?
        .map(|_| function.definition))
}
