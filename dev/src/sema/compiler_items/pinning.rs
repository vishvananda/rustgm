//! Pin construction retains the real inherent source body and ordinary call checks.
use crate::{
    resolve::{expansion::Report, model::DefKind, package::Error},
    sema::{associated::OwnerKind, model::Declarations},
    support::bindings::LangItem,
};

#[cfg(test)]
#[path = "../../../tests/invariants/pin_headers.rs"]
mod tests;

pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    // Imported source contracts require their own authenticated transport.
    if report.package.source_core().is_none() {
        return Ok(());
    }
    let Some(declaration) = catalog.language(LangItem::PinNewUnchecked) else {
        return Ok(());
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 4)?;
    catalog.require_provider_report(report).map_err(error)?;
    let definition = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or_else(|| error("source pin constructor lost its actual definition"))?;
    let owner = facts
        .associated
        .owner(declaration.definition)
        .and_then(|id| facts.associated.get(id))
        .ok_or_else(|| error("source pin constructor lacks its actual inherent owner"))?;
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
            "source pin constructor changed its inherent declaration identity",
        ));
    }
    let function = facts
        .function_header(declaration.definition)
        .map_err(error)?
        .ok_or_else(|| error("source pin constructor lacks its actual callable header"))?;
    if function.declaration.source() != Some(declaration.origin)
        || !declaration_signature(facts, declaration.definition, function.signature)
        || function.body.is_none()
    {
        return Err(error(
            "source pin constructor changed its actual signature or body",
        ));
    }
    // The role's target imposes no additional signature/body restrictions.
    // Ordinary calls still execute and prove this actual body. An implicit pin
    // consumer must separately authenticate its Pin owner and exact operands.
    super::roles::complete_pin_constructor(facts, declaration)
}

fn declaration_signature(
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
