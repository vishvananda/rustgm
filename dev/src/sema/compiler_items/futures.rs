//! Future declaration identity is separate from demanded polling and state machines.
use super::Catalog;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
    },
    sema::model::Declarations,
    support::bindings::LangItem,
    types::ItemId,
};

/// Readiness for async result formation, before the final source role ledger joins.
/// No method body, poll ABI, pinning or executable future is supplied here.
pub(in crate::sema) fn signature_roles(
    report: &Report,
    facts: &Declarations,
) -> Result<(DefId, DefId), Error> {
    crate::sema::header_attributes::charge_source_contract_query(facts, 3)?;
    let catalog = facts.source_bindings.as_deref().ok_or_else(|| {
        Error::unlocated("async signatures require genuine source Future declarations")
    })?;
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    let future = catalog
        .language(LangItem::Future)
        .ok_or_else(|| Error::unlocated("async signatures require the actual Future trait"))?;
    let output = catalog
        .language(LangItem::FutureOutput)
        .ok_or_else(|| Error::unlocated("async signatures require the actual Future Output"))?;
    let error = |message| Declarations::error(report, future.origin, message);
    super::headers::Trait::source(report, facts, future).map_err(error)?;
    if output.parent != Some(future.definition)
        || facts
            .types
            .projection_schema(ItemId(output.definition.0))
            .is_none_or(|schema| schema.trait_item.0 != future.definition.0)
    {
        return Err(error("async Future Output belongs to another actual trait"));
    }
    Ok((future.definition, output.definition))
}

/// Validate independently present source declarations without inventing a poll ABI.
/// Alternative unused language members remain ordinary checked source declarations.
pub(super) fn complete(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
) -> Result<[Option<LangItem>; 3], Error> {
    let mut completed = [None; 3];
    for (slot, role) in [
        LangItem::Future,
        LangItem::FutureOutput,
        LangItem::FuturePoll,
    ]
    .into_iter()
    .enumerate()
    {
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        crate::sema::header_attributes::charge_source_contract_query(facts, 2)?;
        let error = |message| Declarations::error(report, declaration.origin, message);
        catalog.require_provider_report(report).map_err(error)?;
        let definition = report
            .resolution
            .definitions
            .get(declaration.definition.0 as usize)
            .ok_or_else(|| error("source Future member lost its declaration"))?;
        if definition.node != Some(declaration.origin) {
            return Err(error(
                "source Future member changed its declaration identity",
            ));
        }
        match role {
            LangItem::Future => {
                super::headers::Trait::source(report, facts, declaration).map_err(error)?;
            }
            LangItem::FutureOutput => {
                let schema = facts
                    .types
                    .projection_schema(ItemId(declaration.definition.0))
                    .ok_or_else(|| {
                        error("source future_output lacks its actual projection schema")
                    })?;
                if definition.kind != DefKind::TypeAlias
                    || declaration.parent != Some(DefId(schema.trait_item.0))
                {
                    return Err(error(
                        "source future_output changed its actual associated owner",
                    ));
                }
            }
            LangItem::FuturePoll => {
                let function = facts
                    .function_header(declaration.definition)
                    .map_err(error)?
                    .ok_or_else(|| error("source poll lacks its actual callable declaration"))?;
                if function.declaration.source() != Some(declaration.origin)
                    || definition.kind != DefKind::Function
                {
                    return Err(error("source poll changed its actual callable identity"));
                }
            }
            _ => return Err(error("invalid source Future role completion")),
        }
        completed[slot] = Some(role);
    }
    Ok(completed)
}
