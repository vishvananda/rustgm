//! Actual drop glue is a checked source contract, never its placeholder body.
use super::{
    Catalog, Declaration, Declarations, DefKind, Error, Kind, LangItem, Report, TypeId, function,
};
use crate::{
    support::constness::Constness,
    types::{arguments::Argument, callable::Convention, parameters::clauses::Clause},
};

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    completed: u32,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let definition = &report.resolution.definitions[declaration.definition.0 as usize];
    if declaration.parent.is_some()
        || definition.node != Some(declaration.origin)
        || definition.kind != DefKind::Function
        || !facts.is_const_function(declaration.definition)
    {
        return Err(error(
            "source drop_glue requires an actual const module function",
        ));
    }
    let function = function(facts, declaration.definition)
        .ok_or_else(|| error("source drop_glue signature is not prepared"))?;
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: true,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err(error(
            "source drop_glue requires its const unsafe Rust signature",
        ));
    };
    let [parameter] = facts
        .generics
        .parameters
        .get(&declaration.definition)
        .map_or(&[][..], Vec::as_slice)
    else {
        return Err(error("source drop_glue requires one type parameter"));
    };
    let Argument::Type(element) = parameter.argument else {
        return Err(error("source drop_glue parameter must be a type"));
    };
    if parameter.anonymous
        || parameter.default.is_some()
        || !matches!(facts.types.kind(element), Some(Kind::Parameter(_)))
    {
        return Err(error(
            "source drop_glue type parameter changed its contract",
        ));
    }
    let [input] = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("source drop_glue inputs are missing"))?
    else {
        return Err(error("source drop_glue requires one unique reference"));
    };
    if !matches!(facts.types.kind(*input), Some(Kind::Reference { mutable: true, referent, region }) if referent == element && region.bound_parts() == Some((0, 0)))
    {
        return Err(error("source drop_glue requires its exact &mut T input"));
    }
    requirements(report, facts, catalog, declaration, completed, element)
}

fn requirements(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    completed: u32,
    element: TypeId,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    if completed & (1 << LangItem::Destruct.index()) == 0 || catalog.sizing.is_none() {
        return Err(error(
            "source drop_glue requires completed actual sizing and Destruct declarations",
        ));
    }
    let mut pointee = false;
    let mut destruct = false;
    for predicate in facts
        .generics
        .declared_predicates_in_scope(declaration.definition)
    {
        let Clause::Predicate(predicate) = predicate.value else {
            return Err(error(
                "source drop_glue has unsupported quantified requirements",
            ));
        };
        let Some((application, effect)) = predicate.bound.trait_parts() else {
            return Err(error(
                "source drop_glue has unsupported defining requirements",
            ));
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or_else(|| error("source drop_glue requirement is not prepared"))?;
        if predicate.ty != element
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err(error("source drop_glue requirement changed its parameter"));
        }
        let role = if effect == Constness::Ordinary {
            LangItem::PointeeSized
        } else if effect == Constness::Conditional {
            LangItem::Destruct
        } else {
            return Err(error(
                "source drop_glue requirement has the wrong const qualification",
            ));
        };
        if catalog
            .language(role)
            .is_none_or(|d| d.definition.0 != application.item.0)
        {
            return Err(error(
                "source drop_glue requirement names another declaration",
            ));
        }
        let seen = if role == LangItem::PointeeSized {
            &mut pointee
        } else {
            &mut destruct
        };
        if std::mem::replace(seen, true) {
            return Err(error("source drop_glue repeats a defining requirement"));
        }
    }
    if !pointee || !destruct {
        return Err(error(
            "source drop_glue requires actual PointeeSized and Conditional Destruct",
        ));
    }
    Ok(())
}

/// Exact target classification for the later proved cleanup consumer. A matching
/// reserved-but-unvalidated helper is an error, never an ordinary fallback body.
pub(crate) fn member(
    facts: &Declarations,
    definition: crate::resolve::model::DefId,
) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some(declaration) = catalog.language(LangItem::DropGlue) else {
        return Ok(false);
    };
    if declaration.definition != definition {
        return Ok(false);
    }
    catalog.validated_language(LangItem::DropGlue)?;
    Ok(true)
}
