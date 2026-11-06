//! Completed source value roles retain ordinary nominal and variant identities.
use crate::{
    resolve::{bindings::Declaration, expansion::Report, model::DefKind, package::Error},
    sema::{header_attributes, model::Declarations},
    support::bindings::LangItem,
    types::{ItemId, Kind, aggregate},
};

const ROLES: [LangItem; 7] = [
    LangItem::Poll,
    LangItem::PollReady,
    LangItem::PollPending,
    LangItem::Context,
    LangItem::CoroutineState,
    LangItem::ResumeTy,
    LangItem::CStr,
];

/// Complete ordinary headers/fields without restricting Rust-valid unused shapes.
pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    for role in ROLES.into_iter().chain(super::interval_values::NOMINALS) {
        let Some(declaration) = facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.language(role))
        else {
            continue;
        };
        let error = |message| Declarations::error(report, declaration.origin, message);
        header_attributes::charge_source_contract_query(facts, 4)?;
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| error("source value declaration lost its catalog"))?;
        catalog.require_provider_report(report).map_err(error)?;
        let definition = report
            .resolution
            .definitions
            .get(declaration.definition.0 as usize)
            .ok_or_else(|| error("source value declaration lost its actual definition"))?;
        if definition.node != Some(declaration.origin) {
            return Err(error(
                "source value declaration changed its actual source identity",
            ));
        }
        if matches!(role, LangItem::PollReady | LangItem::PollPending) {
            variant(facts, declaration, definition.kind).map_err(error)?;
        } else {
            nominal(facts, declaration, role).map_err(error)?;
        }
        super::roles::complete_value_declaration(facts, role, declaration)?;
    }
    Ok(())
}

fn nominal(
    facts: &Declarations,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), &'static str> {
    let item = ItemId(declaration.definition.0);
    let expected = if matches!(role, LangItem::Poll | LangItem::CoroutineState) {
        aggregate::Kind::Enum
    } else {
        aggregate::Kind::Struct
    };
    if !matches!(facts.definitions.get(item.0 as usize).copied().flatten().and_then(|ty| facts.types.kind(ty)), Some(Kind::Adt(owner, _)) if owner == item)
        || !facts
            .generics
            .nominal_contracts
            .contains(&declaration.definition)
        || facts
            .aggregates
            .get(item)
            .is_none_or(|definition| definition.kind != expected)
    {
        return Err("source value declaration lacks its actual completed nominal shape");
    }
    Ok(())
}

fn variant(
    facts: &Declarations,
    declaration: Declaration,
    kind: DefKind,
) -> Result<(), &'static str> {
    let id = facts
        .members
        .at_node(declaration.origin)
        .ok_or("polling variant lacks its actual field producer")?;
    let variant = facts
        .members
        .variants
        .get(id.0 as usize)
        .ok_or("polling variant lost its actual member row")?;
    if kind != DefKind::Variant
        || variant.origin.source() != Some(declaration.origin)
        || facts
            .members
            .variants(variant.owner)
            .get(variant.ordinal as usize)
            != Some(&id)
        || facts
            .aggregates
            .get(variant.owner)
            .is_none_or(|owner| owner.kind != aggregate::Kind::Enum)
    {
        return Err("polling variant changed its actual owner/member identity");
    }
    Ok(())
}
