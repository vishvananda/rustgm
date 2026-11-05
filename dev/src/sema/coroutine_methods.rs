//! Genuine authored futures borrow their defining clauses and actual source protocol.
mod polling;
mod protocol;
mod schema;
pub(crate) use polling::FutureMethod;
pub(in crate::sema) use polling::{concrete_self, retain};
#[cfg(test)]
#[path = "../../tests/invariants/coroutine_evidence.rs"]
mod tests;
use crate::{
    resolve::model::DefId,
    sema::{model::Declarations, projections::families},
    support::bindings::LangItem,
    types::{GenericArgs, ItemId, Kind, TypeId, projection::Application, substitute::Bound},
};

pub(in crate::sema) fn future_role(facts: &Declarations) -> Result<Option<DefId>, &'static str> {
    facts
        .source_bindings
        .as_deref()
        .map(|catalog| catalog.role(LangItem::Future))
        .transpose()
        .map(|role| role.flatten().map(|role| role.definition))
}

/// One captured fold maps the real function's canonical input ledger.
pub(in crate::sema) fn inputs(
    facts: &mut Declarations,
    future: TypeId,
) -> Result<families::Requirements, &'static str> {
    facts.checked_future_definition(future)?;
    schema::require(facts)?;
    let Some(Kind::Future {
        identity,
        arguments,
    }) = facts.types.kind(future)
    else {
        return Err("future formation requires its genuine full application");
    };
    let application = facts
        .opaques
        .declarations
        .get(&identity)
        .ok_or("future formation lost its actual opaque owner")?
        .application;
    let Some(Kind::Opaque(actual, formal)) = facts.types.kind(application) else {
        return Err("future formation lost its actual formal capture list");
    };
    if actual != identity {
        return Err("future formation changed its actual declaration identity");
    }
    let mut fold =
        Bound::captured_arguments_charged(&facts.types, formal, arguments, &|work, bytes| {
            facts.charge_caller_inputs(work, bytes)
        })?;
    facts.authored_future_inputs(future, &mut fold)
}

/// Only the exact complete Output projection may use this selected Future proof.
pub(in crate::sema) fn output(
    facts: &Declarations,
    application: Application,
    future: TypeId,
) -> Result<TypeId, &'static str> {
    facts.checked_future_definition(future)?;
    schema::require(facts)?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("future Output requires its actual source catalog")?;
    let trait_ = catalog
        .validated_language(LangItem::Future)?
        .ok_or("future Output lost its completed actual Future trait")?;
    let member = catalog
        .validated_language(LangItem::FutureOutput)?
        .ok_or("future Output lost its completed actual Output member")?;
    let schema = facts
        .types
        .projection_schema(ItemId(member.definition.0))
        .ok_or("future Output lost its canonical projection schema")?;
    let trait_application = facts
        .types
        .trait_application(application.trait_application)
        .ok_or("future Output lost its actual parent application")?;
    if application.subject != future
        || application.item != ItemId(member.definition.0)
        || application.arguments != GenericArgs::EMPTY
        || trait_application.item != ItemId(trait_.definition.0)
        || trait_application.arguments != GenericArgs::EMPTY
        || member.parent != Some(trait_.definition)
        || schema.trait_item != ItemId(trait_.definition.0)
    {
        return Err("future evidence differs from its actual Output subject, trait or member");
    }
    let Some(Kind::Future {
        identity,
        arguments,
    }) = facts.types.kind(future)
    else {
        return Err("future Output lost its genuine full application");
    };
    Ok(facts.types.future_application(identity, arguments)?.output)
}
