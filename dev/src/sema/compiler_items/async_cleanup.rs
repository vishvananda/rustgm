//! `AsyncDrop` declaration identity is independent of demanded cleanup operations.
use super::Catalog;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::{GenericArgs, ItemId, Kind, TypeId, arguments::Argument, callable::Convention},
};

pub(super) fn complete(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
) -> Result<Option<LangItem>, Error> {
    let Some(declaration) = catalog.language(LangItem::AsyncDrop) else {
        return Ok(None);
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    crate::sema::header_attributes::charge_source_contract_query(facts, 2)?;
    catalog.require_provider_report(report).map_err(error)?;
    let header = super::headers::Trait::source(report, facts, declaration).map_err(error)?;
    header.canonical_inputs(facts).map_err(error)?;
    Ok(Some(LangItem::AsyncDrop))
}

pub(in crate::sema) struct Operation {
    pub trait_item: DefId,
    pub member: DefId,
}

/// Only implicit destruction demands the actual first-member protocol. Merely
/// declaring or explicitly calling an alternate trait uses ordinary Rust checks.
pub(in crate::sema) fn operation(
    report: &Report,
    facts: &Declarations,
) -> Result<Option<Operation>, Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    let Some(declaration) = catalog
        .validated_language(LangItem::AsyncDrop)
        .map_err(Error::unlocated)?
    else {
        return Ok(None);
    };
    let error = |message| Declarations::error(report, declaration.origin, message);
    let header = super::headers::Trait::source(report, facts, declaration).map_err(error)?;
    header.canonical_inputs(facts).map_err(error)?;
    crate::sema::header_attributes::charge_source_contract_query(facts, 8 + header.members.len())?;
    let member = *header
        .members
        .first()
        .ok_or_else(|| error("demanded AsyncDrop has no actual first member"))?;
    if !header.parameters.is_empty() || header.captures != 0 {
        return Err(error(
            "demanded AsyncDrop requires its unparameterized trait contract",
        ));
    }
    let function = facts
        .function_header(member)
        .map_err(error)?
        .ok_or_else(|| error("demanded AsyncDrop first member is not an actual function"))?;
    if facts.is_const_function(member)
        || facts
            .generics
            .parameters
            .get(&member)
            .is_some_and(|p| !p.is_empty())
    {
        return Err(error(
            "demanded AsyncDrop member must be non-const without method parameters",
        ));
    }
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
        ..
    }) = facts.types.kind(function.signature)
    else {
        return Err(error(
            "demanded AsyncDrop requires an actual safe Rust function",
        ));
    };
    let pin = catalog
        .validated_language(LangItem::Pin)
        .map_err(error)?
        .ok_or_else(|| error("demanded AsyncDrop requires the actual Pin declaration"))?;
    let [input] = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("missing AsyncDrop receiver inputs"))?
    else {
        return Err(error(
            "demanded AsyncDrop requires exactly one actual receiver",
        ));
    };
    if !receiver(facts, *input, ItemId(pin.definition.0), header.self_type) {
        return Err(error(
            "demanded AsyncDrop requires exact Pin<&mut Self> receiver",
        ));
    }
    if !future_result(facts, catalog, result, member).map_err(error)? {
        return Err(error(
            "demanded AsyncDrop requires its actual Future<Output=()> result",
        ));
    }
    Ok(Some(Operation {
        trait_item: declaration.definition,
        member,
    }))
}

fn receiver(facts: &Declarations, input: TypeId, pin: ItemId, subject: TypeId) -> bool {
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(input) else {
        return false;
    };
    if item != pin {
        return false;
    }
    let Some(mut arguments) = facts.types.arguments(arguments) else {
        return false;
    };
    let Some(Argument::Type(pointer)) = arguments.next() else {
        return false;
    };
    arguments.next().is_none()
        && matches!(facts.types.kind(pointer), Some(Kind::Reference { mutable: true, referent, region }) if referent == subject && region.bound_parts() == Some((0,0)))
}

fn future_result(
    facts: &Declarations,
    catalog: &Catalog,
    result: TypeId,
    member: DefId,
) -> Result<bool, &'static str> {
    use crate::types::parameters::{clauses::Clause, predicates::Bound};
    let Some(Kind::Opaque(id, _)) = facts.types.kind(result) else {
        return Ok(false);
    };
    let Some(opaque) = facts.opaques.declarations.get(&id) else {
        return Ok(false);
    };
    if opaque.owner != member || opaque.application != result {
        return Ok(false);
    }
    let future = catalog
        .validated_language(LangItem::Future)?
        .ok_or("AsyncDrop requires actual Future")?;
    let output = catalog
        .validated_language(LangItem::FutureOutput)?
        .ok_or("AsyncDrop requires actual Future Output")?;
    let mut has_future = false;
    let mut has_output = false;
    for clause in &opaque.bounds {
        let Clause::Predicate(predicate) = *clause else {
            continue;
        };
        match predicate.bound {
            Bound::Trait(application) if predicate.ty == result => {
                has_future |= facts.types.trait_application(application).is_some_and(|a| {
                    a.item.0 == future.definition.0 && a.arguments == GenericArgs::EMPTY
                });
            }
            Bound::ProjectionEqual(TypeId::UNIT) => {
                has_output |= matches!(facts.types.kind(predicate.ty), Some(Kind::Projection(p)) if p.item.0 == output.definition.0 && p.subject == result && p.arguments == GenericArgs::EMPTY && facts.types.trait_application(p.trait_application).is_some_and(|a|a.item.0 == future.definition.0 && a.arguments == GenericArgs::EMPTY));
            }
            _ => {}
        }
    }
    Ok(has_future && has_output && output.parent == Some(future.definition))
}
