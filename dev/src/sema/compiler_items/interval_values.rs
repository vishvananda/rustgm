//! Range tags authenticate ordinary source structs and a real inherent constructor.
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
};

pub(super) const NOMINALS: [LangItem; 10] = [
    LangItem::RangeFrom,
    LangItem::RangeFull,
    LangItem::RangeInclusiveStruct,
    LangItem::Range,
    LangItem::RangeToInclusive,
    LangItem::RangeTo,
    LangItem::RangeFromCopy,
    LangItem::RangeCopy,
    LangItem::RangeInclusiveCopy,
    LangItem::RangeToInclusiveCopy,
];

pub(in crate::sema) fn complete_constructor(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let role = LangItem::RangeInclusiveNew;
    let Some(declaration) = super::headers::functions::inherent(report, facts, role)? else {
        return Ok(());
    };
    super::roles::complete_constructor(facts, role, declaration)
}

/// Authenticate the implicit legacy call; its actual body owns all storage.
pub(in crate::sema) fn source_constructor(
    report: &Report,
    facts: &Declarations,
    site: crate::resolve::unit::NodeKey,
) -> Result<crate::resolve::model::DefId, Error> {
    let error = |message| Declarations::error(report, site, message);
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| error("inclusive constructor requires its actual source catalog"))?;
    catalog.require_provider_report(report).map_err(error)?;
    crate::sema::header_attributes::charge_source_contract_query(facts, 5)?;
    let nominal = catalog
        .validated_language(LangItem::RangeInclusiveStruct)
        .map_err(error)?
        .ok_or_else(|| error("inclusive syntax requires its actual source range declaration"))?;
    crate::sema::header_attributes::completed_source_image(facts, report, nominal.definition)?;
    // Recheck the current nominal producer facts as well as its completed role.
    if report
        .resolution
        .definitions
        .get(nominal.definition.0 as usize)
        .and_then(|definition| definition.node)
        != Some(nominal.origin)
    {
        return Err(error("inclusive range changed its actual nominal source"));
    }
    super::role_values::nominal(facts, nominal, LangItem::RangeInclusiveStruct).map_err(error)?;
    let declaration =
        super::headers::functions::inherent(report, facts, LangItem::RangeInclusiveNew)?
            .ok_or_else(|| error("inclusive syntax requires its actual source constructor"))?;
    crate::sema::header_attributes::completed_source_image(facts, report, declaration.definition)?;
    let header = facts
        .function_header(declaration.definition)
        .map_err(error)?
        .ok_or_else(|| error("inclusive constructor requires its completed header"))?;
    let Some(crate::types::Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(header.signature)
    else {
        return Err(error(
            "inclusive constructor requires its actual function signature",
        ));
    };
    if facts.types.list(parameters).map(<[_]>::len) != Some(2)
        || !matches!(facts.types.kind(result), Some(crate::types::Kind::Adt(owner, _))
            if owner.0 == nominal.definition.0)
    {
        return Err(error(
            "inclusive constructor requires two inputs and its actual range result",
        ));
    }
    Ok(declaration.definition)
}

#[cfg(test)]
#[path = "../../../tests/invariants/inclusive_calls.rs"]
mod tests;
