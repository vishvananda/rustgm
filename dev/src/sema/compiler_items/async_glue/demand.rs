//! Closed helper targets are data, not permission to execute a written body.
#[cfg(test)]
#[path = "../../../../tests/invariants/future_targets.rs"]
mod tests;
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{Kind, TypeId, arguments::Argument, instance::Instance},
};

/// Select the exact compiler helper and its closed target. Direct nominal
/// cleanup additionally authenticates the actual `AsyncDrop` operation. Absence of
/// that direct record says nothing about recursive fields or synchronous fallback.
pub(crate) fn target(
    report: &Report,
    facts: &Declarations,
    instance: Instance,
) -> Result<Option<TypeId>, Error> {
    let Some(Kind::FunctionItem {
        item, arguments, ..
    }) = facts.types.kind(instance.ty())
    else {
        return Ok(None);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(declaration) = catalog.language(LangItem::AsyncDropInPlace) else {
        return Ok(None);
    };
    if declaration.definition.0 != item.0 {
        return Ok(None);
    }
    let error = |message| Declarations::error(report, declaration.origin, message);
    catalog
        .validated_language(LangItem::AsyncDropInPlace)
        .map_err(error)?;
    super::complete(report, facts, catalog)?;
    catalog
        .require_async_destructor_registration()
        .map_err(error)?;
    catalog
        .require_source_destructors_prepared()
        .map_err(error)?;
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or_else(|| error("async drop helper lost its selected arguments"))?;
    let Some(Argument::Type(element)) = arguments.next() else {
        return Err(error("async drop helper requires its selected target type"));
    };
    if arguments.next().is_some()
        || facts.types.contains_parameters(element) != Some(false)
        || facts.types.contains_projections(element) != Some(false)
    {
        return Err(error(
            "async drop helper requires a closed normalized target",
        ));
    }
    let function = facts
        .function_indices
        .get(&declaration.definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or_else(|| error("async drop helper lost its actual declaration body"))?;
    super::signature(report, facts, function)?;
    if let Some(Kind::Adt(nominal, _)) = facts.types.kind(element)
        && facts.destructors.async_item(nominal).is_some()
    {
        let operation = super::super::async_cleanup::operation(report, facts)?
            .ok_or_else(|| error("registered async destructor lost its actual source trait"))?;
        if facts.associated.trait_owner(operation.member) != Some(operation.trait_item) {
            return Err(error(
                "async destructor operation changed its actual member owner",
            ));
        }
    }
    Ok(Some(element))
}
