//! Compiler-owned async cleanup never executes the helper's written placeholder.
mod demand;
mod recipes;
use super::Catalog;
use crate::{
    resolve::{expansion::Report, model::DefKind, package::Error},
    sema::model::{Declarations, Function},
    support::bindings::LangItem,
    types::{Kind, TypeId, arguments::Argument, callable::Convention},
};
pub(crate) use demand::target as source_async_drop_glue;
pub(crate) use recipes::recipe;

/// Unused alternate helper declarations retain ordinary source validity. Exact
/// constructor/poll requirements belong to the selected generated operation.
pub(super) fn complete(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
) -> Result<Option<LangItem>, Error> {
    let Some(declaration) = catalog.language(LangItem::AsyncDropInPlace) else {
        return Ok(None);
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 2)?;
    catalog.require_provider_report(report).map_err(error)?;
    let actual = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or_else(|| error("async drop helper lost its declaration identity"))?;
    let header = facts
        .function_header(declaration.definition)
        .map_err(error)?
        .ok_or_else(|| error("async drop helper lacks its actual function header"))?;
    if declaration.parent.is_some()
        || actual.kind != DefKind::Function
        || actual.node != Some(declaration.origin)
        || header.declaration.source() != Some(declaration.origin)
    {
        return Err(error(
            "async drop helper changed its actual module declaration",
        ));
    }
    Ok(Some(LangItem::AsyncDropInPlace))
}

/// Deny eager publication before touching Prepared/CFG state. Matching names and
/// the empty authored body never authorize an ordinary implementation fallback.
pub(crate) fn require_runtime_body(
    report: &Report,
    facts: &Declarations,
    function: &Function,
) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let Some(declaration) = catalog.language(LangItem::AsyncDropInPlace) else {
        return Ok(());
    };
    if declaration.definition != function.definition {
        return Ok(());
    }
    let error = |message| Declarations::error(report, function.node, message);
    catalog
        .validated_language(LangItem::AsyncDropInPlace)
        .map_err(error)?;
    complete(report, facts, catalog)?;
    if declaration.origin != function.node {
        return Err(error(
            "async drop helper body belongs to another source owner",
        ));
    }
    signature(report, facts, function)?;
    catalog
        .require_source_destructors_prepared()
        .map_err(error)?;
    catalog
        .require_async_destructor_registration()
        .map_err(error)?;
    Err(error(
        "compiler-owned async destruction requires generated future state and polling",
    ))
}

fn signature(report: &Report, facts: &Declarations, function: &Function) -> Result<(), Error> {
    let error = |message| Declarations::error(report, function.node, message);
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        unsafe_: true,
        convention: Convention::RUST,
        ..
    }) = facts.types.kind(function.signature)
    else {
        return Err(error(
            "async drop helper requires its unsafe Rust borrow signature",
        ));
    };
    if facts.async_body_output(report, function)? != Some(TypeId::UNIT) {
        return Err(error(
            "async drop helper requires an actual async unit output",
        ));
    }
    let [parameter] = facts
        .generics
        .parameters
        .get(&function.definition)
        .map_or(&[][..], Vec::as_slice)
    else {
        return Err(error(
            "async drop helper requires its actual type parameter",
        ));
    };
    let Argument::Type(element) = parameter.argument else {
        return Err(error("async drop helper parameter must be a type"));
    };
    let [input] = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("async drop helper lost its declared inputs"))?
    else {
        return Err(error(
            "async drop helper requires its actual unique reference",
        ));
    };
    if parameter.anonymous
        || parameter.default.is_some()
        || !matches!(facts.types.kind(element), Some(Kind::Parameter(_)))
        || !matches!(facts.types.kind(*input), Some(Kind::Reference {mutable: true, referent, region})
            if referent == element && region.bound_parts() == Some((0, 0)))
    {
        return Err(error(
            "async drop helper changed its actual &mut T contract",
        ));
    }
    Ok(())
}
