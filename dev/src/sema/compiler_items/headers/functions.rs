//! Ordinary defined functions retain their authored contracts and body identities.
use super::super::Catalog;
use crate::{
    resolve::{bindings::Declaration, expansion::Report, model::DefKind, package::Error},
    sema::{associated::OwnerKind, function_headers::FunctionHeader, model::Declarations},
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

pub(in crate::sema::compiler_items) fn inherent(
    report: &Report,
    facts: &Declarations,
    role: LangItem,
) -> Result<Option<Declaration>, Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    // Imported source contracts require their own authenticated transport.
    if report.package.source_core().is_none() {
        return Ok(None);
    }
    let Some(declaration) = catalog.language(role) else {
        return Ok(None);
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 4)?;
    catalog.require_provider_report(report).map_err(error)?;
    let definition = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or_else(|| error("source inherent constructor lost its actual definition"))?;
    let owner = facts
        .associated
        .owner(declaration.definition)
        .and_then(|id| facts.associated.get(id))
        .ok_or_else(|| error("source inherent constructor lacks its actual inherent owner"))?;
    if definition.node != Some(declaration.origin)
        || definition.kind != DefKind::Function
        || declaration.parent != Some(owner.self_definition)
        || !matches!(
            owner.kind,
            OwnerKind::Impl {
                trait_type: None,
                ..
            }
        )
    {
        return Err(error(
            "source inherent constructor changed its inherent declaration identity",
        ));
    }
    let function = facts
        .function_header(declaration.definition)
        .map_err(error)?
        .ok_or_else(|| error("source inherent constructor lacks its actual callable header"))?;
    if function.declaration.source() != Some(declaration.origin)
        || !declaration_signature(facts, declaration.definition, function.signature)
        || function.body.is_none()
    {
        return Err(error(
            "source inherent constructor changed its actual signature or body",
        ));
    }
    // Ordinary calls still prove and execute the actual body. An implicit
    // consumer separately authenticates its owner and complete operands.
    Ok(Some(declaration))
}

pub(in crate::sema::compiler_items) fn declaration_signature(
    facts: &Declarations,
    definition: crate::resolve::model::DefId,
    signature: crate::types::TypeId,
) -> bool {
    // A relocated public callable has different early/late coordinates. Compare
    // the actual declaration item and its source/body scheme in the same frame.
    // Empty schemes are real completed producer facts, never lookup fallbacks.
    let declaration = facts
        .definitions
        .get(definition.0 as usize)
        .copied()
        .flatten();
    if !matches!(declaration.and_then(|ty|facts.types.kind(ty)), Some(crate::types::Kind::FunctionItem {item, signature: actual, ..}) if item.0==definition.0 && actual==signature)
    {
        return false;
    }
    // Unused declared names can acquire body slots after signature formation.
    // The real body domain may therefore extend the signature's existing frame.
    facts.lifetimes.functions.get(&definition).is_some_and(|scheme| {
        matches!(facts.types.kind(signature), Some(crate::types::Kind::Function {bound_regions, ..}) if bound_regions<=scheme.total)
    })
}
