//! Ordinary defined functions retain their authored contracts and body identities.
use super::super::Catalog;
use crate::{
    resolve::{bindings::Declaration, expansion::Report, package::Error},
    sema::{function_headers::FunctionHeader, model::Declarations},
    support::bindings::LangItem,
    types::Kind,
};

pub(in crate::sema::compiler_items) fn complete(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    role: LangItem,
) -> Result<Option<LangItem>, Error> {
    let Some(declaration) = catalog.language(role) else {
        return Ok(None);
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 4)?;
    catalog.require_provider_report(report).map_err(error)?;
    let header = current(facts, declaration).map_err(error)?;
    super::authored::function(report, declaration, &header).map_err(error)?;
    Ok(Some(role))
}

pub(in crate::sema::compiler_items) fn current(
    facts: &Declarations,
    declaration: Declaration,
) -> Result<FunctionHeader<'_>, &'static str> {
    let header = facts
        .function_header(declaration.definition)?
        .ok_or("source compiler function lacks its completed header")?;
    if declaration.parent.is_some()
        || header.declaration.source() != Some(declaration.origin)
        || header.body.is_none()
    {
        return Err("source compiler function requires its actual defined free function");
    }
    let Some(Kind::FunctionItem {
        item,
        signature,
        arguments,
    }) = facts
        .definitions
        .get(declaration.definition.0 as usize)
        .copied()
        .flatten()
        .and_then(|ty| facts.types.kind(ty))
    else {
        return Err("source compiler function lost its canonical function item");
    };
    if item.0 != declaration.definition.0
        || signature != header.signature
        || !matches!(facts.types.kind(signature), Some(Kind::Function { .. }))
        || facts.types.arguments(arguments).is_none()
    {
        return Err("source compiler function changed its actual canonical header");
    }
    Ok(header)
}
