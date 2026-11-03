//! Reflection declaration identities supply no type provenance or intrinsic execution.
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::{header_attributes, model::Declarations},
    support::bindings::LangItem,
    types::{ItemId, aggregate::Kind},
};

/// The pinned language items restrict the target to a struct, not its generics or fields.
/// Actual reflection consumers must independently validate the types they construct.
pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    for role in [LangItem::TypeInfo, LangItem::TypeId] {
        let Some(catalog) = facts.source_bindings.as_deref() else {
            return Ok(());
        };
        let Some(declaration) = catalog.language(role) else {
            continue;
        };
        let error = |message| Declarations::error(report, declaration.origin, message);
        catalog.require_provider_report(report).map_err(error)?;
        header_attributes::charge_source_contract_query(facts, 1)?;
        let definition = facts
            .aggregates
            .get(ItemId(declaration.definition.0))
            .ok_or_else(|| {
                error("source reflection role requires its completed nominal declaration")
            })?;
        if definition.kind != Kind::Struct {
            return Err(error(
                "source reflection role requires its actual struct definition",
            ));
        }
        super::roles::complete_nominal(facts, role, declaration)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/reflection_headers.rs"]
mod tests;
