//! Actual unsizing declarations classify structural goals, never pointer permission.
#[cfg(test)]
#[path = "../../../../tests/invariants/coercion_evidence.rs"]
mod tests;
use super::{
    Catalog, Declaration, Declarations, DefId, Error, Kind, LangItem, Report, TypeId, copying,
    item, validate_marker,
};
use crate::{
    support::constness::Constness,
    types::{
        arguments::Argument,
        parameters::{clauses::Clause, predicates::Predicate},
    },
};

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    role: LangItem,
    self_type: TypeId,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let syntax = item(report, declaration)?;
    if !report
        .package
        .unit(declaration.origin.file)
        .configured
        .ast
        .list(syntax.where_clause)
        .is_empty()
    {
        return Err(error(
            "source unsizing role has unsupported defining constraints",
        ));
    }
    copying::require_super(
        report,
        facts,
        catalog,
        declaration,
        self_type,
        if role == LangItem::Unsize {
            LangItem::PointeeSized
        } else {
            LangItem::Sized
        },
        Constness::Ordinary,
    )?;
    validate_target(report, facts, catalog, declaration, role)?;
    validate_members(report, facts, declaration, role)
}

fn validate_target(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let parameters = facts
        .generics
        .parameters
        .get(&declaration.definition)
        .ok_or_else(|| error("source unsizing role has no target parameter"))?;
    let [parameter] = parameters.as_slice() else {
        return Err(error(
            "source unsizing role requires exactly one target parameter",
        ));
    };
    let Argument::Type(target) = parameter.argument else {
        return Err(error("source unsizing target must be a type parameter"));
    };
    if parameter.anonymous
        || parameter.default.is_some()
        || !matches!(facts.types.kind(target), Some(Kind::Parameter(_)))
    {
        return Err(error(
            "source unsizing target changed its parameter contract",
        ));
    }
    let expected = if role == LangItem::DispatchFromDyn {
        // The genuine Sized formal carries the implicit MetaSized source clause.
        LangItem::MetaSized
    } else {
        LangItem::PointeeSized
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(target) else {
        return Err(error(
            "source unsizing target is not its actual type parameter",
        ));
    };
    if facts.types.parameter(id)
        != Some(crate::types::parameters::Parameter {
            owner: crate::types::ItemId(declaration.definition.0),
            index: 0,
            kind: crate::types::parameters::ParameterKind::Type {
                sized: role == LangItem::DispatchFromDyn,
            },
        })
    {
        return Err(error(
            "source unsizing target changed its declaration or sizing identity",
        ));
    }
    let pointee = catalog
        .language(expected)
        .ok_or_else(|| error("source unsizing role requires actual PointeeSized"))?;
    let mut found = false;
    for clause in facts
        .generics
        .declared_predicates_in_scope(declaration.definition)
    {
        let Clause::Predicate(predicate) = clause.value else {
            return Err(error(
                "source unsizing role has unsupported quantified requirements",
            ));
        };
        if predicate.ty != target {
            continue;
        }
        if role == LangItem::DispatchFromDyn
            && predicate.bound == crate::types::parameters::predicates::Bound::Sized
        {
            // Sized is already authenticated on the actual formal. Its builtin
            // marker is distinct from the retained source MetaSized clause.
            continue;
        }
        let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
            return Err(error(
                "source unsizing target requires ordinary PointeeSized",
            ));
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or_else(|| error("source unsizing target requirement is malformed"))?;
        if application.item.0 != pointee.definition.0
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
            || std::mem::replace(&mut found, true)
        {
            return Err(error(
                "source unsizing target changed its exact PointeeSized requirement",
            ));
        }
    }
    if !found {
        return Err(error(if role == LangItem::DispatchFromDyn {
            "source DispatchFromDyn target requires actual MetaSized"
        } else {
            "source unsizing target requires actual PointeeSized"
        }));
    }
    Ok(())
}

fn validate_members(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
    role: LangItem,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    if role == LangItem::Unsize {
        return validate_marker(report, facts, declaration);
    }
    let owner = facts
        .associated
        .owner(declaration.definition)
        .ok_or_else(|| error("source CoerceUnsized owner is missing"))?;
    if facts
        .associated
        .owner_members(owner)
        .is_none_or(|members| !members.is_empty())
    {
        return Err(error("source CoerceUnsized must not declare members"));
    }
    Ok(())
}

/// Classification alone is data. A caller must still prove the full goal.
pub(crate) fn kind(
    facts: &Declarations,
    definition: DefId,
) -> Result<Option<LangItem>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    for role in [
        LangItem::Unsize,
        LangItem::CoerceUnsized,
        LangItem::DispatchFromDyn,
    ] {
        if catalog
            .language(role)
            .is_some_and(|d| d.definition == definition)
        {
            catalog.validated_language(role)?;
            return Ok(Some(role));
        }
    }
    Ok(None)
}

/// Only successful ordinary Unsize selection uses Builtin evidence.
pub(crate) fn predicate(facts: &Declarations, predicate: Predicate) -> Result<bool, &'static str> {
    let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or("invalid source Unsize application")?;
    if kind(facts, DefId(application.item.0))? != Some(LangItem::Unsize) {
        return Ok(false);
    }
    Ok(facts
        .types
        .arguments(application.arguments)
        .is_some_and(|mut args| {
            matches!(args.next(), Some(Argument::Type(_))) && args.next().is_none()
        }))
}
