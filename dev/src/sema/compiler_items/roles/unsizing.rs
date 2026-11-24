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
    let target =
        target_parameter(facts, declaration.definition, parameters, role).map_err(error)?;
    let clauses = facts
        .generics
        .declared_predicates_in_scope(declaration.definition)
        .map(|clause| clause.value);
    target_requirements(facts, catalog, target, role, clauses).map_err(error)
}

fn target_parameter(
    facts: &Declarations,
    definition: DefId,
    parameters: &[crate::sema::generics::Parameter],
    role: LangItem,
) -> Result<TypeId, &'static str> {
    let [parameter] = parameters else {
        return Err("source unsizing role requires exactly one target parameter");
    };
    let Argument::Type(target) = parameter.argument else {
        return Err("source unsizing target must be a type parameter");
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(target) else {
        return Err("source unsizing target is not its actual type parameter");
    };
    if parameter.anonymous
        || parameter.default.is_some()
        || facts.types.parameter(id)
            != Some(crate::types::parameters::Parameter {
                owner: crate::types::ItemId(definition.0),
                index: 0,
                kind: crate::types::parameters::ParameterKind::Type {
                    sized: role == LangItem::DispatchFromDyn,
                },
            })
    {
        return Err("source unsizing target changed its declaration or sizing identity");
    }
    Ok(target)
}

fn target_requirements(
    facts: &Declarations,
    catalog: &Catalog,
    target: TypeId,
    role: LangItem,
    clauses: impl Iterator<Item = Clause>,
) -> Result<(), &'static str> {
    let expected = if role == LangItem::DispatchFromDyn {
        // The genuine Sized formal carries the implicit MetaSized source clause.
        LangItem::MetaSized
    } else {
        LangItem::PointeeSized
    };
    let pointee = catalog
        .role(expected)?
        .ok_or("source unsizing role requires actual PointeeSized")?;
    let mut found = false;
    for clause in clauses {
        let Clause::Predicate(predicate) = clause else {
            return Err("source unsizing role has unsupported quantified requirements");
        };
        if predicate.ty != target
            || role == LangItem::DispatchFromDyn
                && predicate.bound == crate::types::parameters::predicates::Bound::Sized
        {
            continue;
        }
        let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
            return Err("source unsizing target requires ordinary PointeeSized");
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or("source unsizing target requirement is malformed")?;
        if application.item.0 != pointee.definition.0
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
            || std::mem::replace(&mut found, true)
        {
            return Err("source unsizing target changed its exact PointeeSized requirement");
        }
    }
    if !found {
        return Err(if role == LangItem::DispatchFromDyn {
            "source DispatchFromDyn target requires actual MetaSized"
        } else {
            "source unsizing target requires actual PointeeSized"
        });
    }
    Ok(())
}

pub(super) const ROLES: [LangItem; 3] = [
    LangItem::Unsize,
    LangItem::CoerceUnsized,
    LangItem::DispatchFromDyn,
];

pub(super) fn imported_roles(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
    complete: &mut super::RoleSet,
) -> Result<(), &'static str> {
    for role in ROLES {
        if let Some(declaration) = catalog.role(role)? {
            imported(report, facts, catalog, declaration, role, steps)?;
            complete.insert(role);
        }
    }
    Ok(())
}

fn imported(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: super::super::Role,
    role: LangItem,
    steps: &mut usize,
) -> Result<(), &'static str> {
    use super::super::headers::{Flags, Trait};
    use crate::resolve::bindings::HeaderAttribute;
    let header = Trait::current(report, facts, declaration, steps)?;
    header.profile(
        &facts.types,
        Flags {
            unsafe_: false,
            auto_: false,
            constant: false,
        },
        1,
    )?;
    let marker = role == LangItem::Unsize;
    if header.written.parameters != 1
        || !header.members.is_empty()
        || header.attributes.contains(HeaderAttribute::DynIncompatible) != marker
        || facts
            .associated
            .imported_denies_implementation(header.definition)?
            != marker
    {
        return Err("imported unsizing role changed its empty declaration or restrictions");
    }
    let expected = if marker {
        LangItem::PointeeSized
    } else {
        LangItem::Sized
    };
    copying::require_header_super(facts, catalog, &header, expected, Constness::Ordinary)?;
    let target = target_parameter(facts, header.definition, header.parameters, role)?;
    let inputs = header.canonical_inputs(facts)?;
    *steps = steps.saturating_add(3 + inputs.len());
    target_requirements(
        facts,
        catalog,
        target,
        role,
        inputs.iter().map(|input| input.value),
    )
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
    for role in ROLES {
        if catalog
            .role(role)?
            .is_some_and(|d| d.definition == definition)
        {
            catalog.validated_role(role)?;
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
