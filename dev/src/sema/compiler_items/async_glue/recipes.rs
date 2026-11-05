//! A checked compiler operation retains its genuine source owner, never an empty CFG.
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey},
    sema::model::{Declarations, Function},
    support::bindings::LangItem,
    types::{GenericArgs, Kind, TypeId, opaque::OpaqueId},
};

/// Borrowed declaration data only. There is no executable body or hidden storage.
pub(crate) struct Recipe<'a> {
    pub function: &'a Function,
    pub body: NodeKey,
    pub future: TypeId,
    pub opaque: OpaqueId,
    pub captures: GenericArgs,
    pub output: TypeId,
    pub parameters: &'a [TypeId],
}

pub(crate) fn recipe<'a>(
    report: &Report,
    facts: &'a Declarations,
    definition: DefId,
) -> Result<Option<Recipe<'a>>, Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(declaration) = catalog.language(LangItem::AsyncDropInPlace) else {
        return Ok(None);
    };
    if declaration.definition != definition {
        return Ok(None);
    }
    let error = |message| Declarations::error(report, declaration.origin, message);
    catalog
        .validated_language(LangItem::AsyncDropInPlace)
        .map_err(error)?;
    super::complete(report, facts, catalog)?;
    catalog
        .require_source_destructors_prepared()
        .map_err(error)?;
    catalog
        .require_async_destructor_registration()
        .map_err(error)?;
    let function = facts
        .function_indices
        .get(&definition)
        .and_then(|&index| facts.functions.get(index))
        .ok_or_else(|| error("compiler operation lost its actual source function"))?;
    if function.definition != definition || function.node != declaration.origin {
        return Err(error("compiler operation changed its actual source owner"));
    }
    super::signature(report, facts, function)?;
    let body = function
        .body
        .ok_or_else(|| error("compiler operation requires its actual authored body"))?;
    if !facts.typed_function_bodies.contains(&definition) {
        return Err(error(
            "compiler operation requires its completed authored body checking",
        ));
    }
    let Some(Kind::Function {
        parameters,
        result: future,
        ..
    }) = facts.types.kind(function.signature)
    else {
        return Err(error(
            "compiler operation lost its actual constructor signature",
        ));
    };
    let Some(Kind::Opaque(opaque, captures)) = facts.types.kind(future) else {
        return Err(error(
            "compiler operation requires its genuine async opaque result",
        ));
    };
    let contract = facts
        .opaques
        .declarations
        .get(&opaque)
        .ok_or_else(|| error("compiler operation lost its actual opaque contract"))?;
    if contract.owner != definition
        || contract.origin.source() != Some(function.node)
        || contract.application != future
        || facts
            .opaques
            .owners
            .get(&definition)
            .is_none_or(|ids| !ids.contains(&opaque))
    {
        return Err(error(
            "compiler operation changed its opaque owner or capture application",
        ));
    }
    let parameters = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("compiler operation lost its actual formal inputs"))?;
    Ok(Some(Recipe {
        function,
        body,
        future,
        opaque,
        captures,
        output: TypeId::UNIT,
        parameters,
    }))
}
