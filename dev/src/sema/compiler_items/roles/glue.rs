//! Actual drop glue is a checked source contract, never its placeholder body.
use super::{
    Catalog, Declaration, Declarations, DefKind, Error, Kind, LangItem, Report, RoleSet, TypeId,
    function,
};
use crate::{
    resolve::model::DefId,
    support::constness::Constness,
    types::{arguments::Argument, callable::Convention, parameters::clauses::Clause},
};

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    completed: RoleSet,
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
    let element = signature(facts, declaration.definition, function.signature).map_err(error)?;
    requirements(
        facts,
        catalog,
        completed,
        element,
        facts
            .generics
            .declared_predicates_in_scope(declaration.definition)
            .map(|input| input.value),
    )
    .map_err(error)
}

pub(super) fn imported(
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
    complete: &mut RoleSet,
) -> Result<(), &'static str> {
    let Some(declaration) = catalog.role(LangItem::DropGlue)? else {
        return Ok(());
    };
    let actual = facts
        .declaration_header(declaration.definition)?
        .ok_or("imported drop_glue lost its actual declaration header")?;
    if declaration.parent.is_some()
        || actual.declaration != declaration.origin
        || !matches!(
            actual.body,
            Some(crate::resolve::body_origin::BodyOrigin::Imported(_))
        )
        || !facts.is_const_function(declaration.definition)
    {
        return Err("imported drop_glue requires its actual defined const module function");
    }
    let element = signature(facts, declaration.definition, actual.signature)?;
    let inputs = crate::sema::declaration_inputs::declared_own(
        &facts.generics,
        facts.crates.as_deref(),
        declaration.definition,
    )?;
    *steps = steps.saturating_add(8 + inputs.len());
    requirements(
        facts,
        catalog,
        *complete,
        element,
        inputs.iter().map(|input| input.value),
    )?;
    complete.insert(LangItem::DropGlue);
    Ok(())
}

fn signature(
    facts: &Declarations,
    definition: DefId,
    signature: TypeId,
) -> Result<TypeId, &'static str> {
    let Some(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: true,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err("source drop_glue requires its const unsafe Rust signature");
    };
    let Some([parameter]) = facts
        .generics
        .parameters
        .get(&definition)
        .map(Vec::as_slice)
    else {
        return Err("source drop_glue requires one type parameter");
    };
    let Argument::Type(element) = parameter.argument else {
        return Err("source drop_glue parameter must be a type");
    };
    if parameter.anonymous
        || parameter.default.is_some()
        || !matches!(facts.types.kind(element), Some(Kind::Parameter(id)) if facts.types.parameter(id)
            .is_some_and(|parameter| parameter.owner.0 == definition.0 && parameter.index == 0))
    {
        return Err("source drop_glue type parameter changed its contract");
    }
    let Some([input]) = facts.types.list(parameters) else {
        return Err("source drop_glue requires one unique reference");
    };
    if !matches!(facts.types.kind(*input), Some(Kind::Reference { mutable: true, referent, region }) if referent == element && region.bound_parts() == Some((0, 0)))
    {
        return Err("source drop_glue requires its exact &mut T input");
    }
    Ok(element)
}

fn requirements(
    facts: &Declarations,
    catalog: &Catalog,
    completed: RoleSet,
    element: TypeId,
    inputs: impl Iterator<Item = Clause>,
) -> Result<(), &'static str> {
    if !completed.contains(LangItem::Destruct) || catalog.sizing.is_none() {
        return Err("source drop_glue requires completed actual sizing and Destruct declarations");
    }
    let mut pointee = false;
    let mut destruct = false;
    for clause in inputs {
        let Clause::Predicate(predicate) = clause else {
            return Err("source drop_glue has unsupported quantified requirements");
        };
        let Some((application, effect)) = predicate.bound.trait_parts() else {
            return Err("source drop_glue has unsupported defining requirements");
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or("source drop_glue requirement is not prepared")?;
        if predicate.ty != element
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err("source drop_glue requirement changed its parameter");
        }
        let (role, seen) = match effect {
            Constness::Ordinary => (LangItem::PointeeSized, &mut pointee),
            Constness::Conditional => (LangItem::Destruct, &mut destruct),
            Constness::Const => {
                return Err("source drop_glue requirement has the wrong const qualification");
            }
        };
        if catalog
            .role(role)?
            .is_none_or(|d| d.definition.0 != application.item.0)
        {
            return Err("source drop_glue requirement names another declaration");
        }
        if std::mem::replace(seen, true) {
            return Err("source drop_glue repeats a defining requirement");
        }
    }
    if !pointee || !destruct {
        return Err("source drop_glue requires actual PointeeSized and Conditional Destruct");
    }
    Ok(())
}

/// Exact target classification for the later proved cleanup consumer. A matching
/// reserved-but-unvalidated helper is an error, never an ordinary fallback body.
pub(crate) fn member(facts: &Declarations, definition: DefId) -> Result<bool, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some(declaration) = catalog.role(LangItem::DropGlue)? else {
        return Ok(false);
    };
    if declaration.definition != definition {
        return Ok(false);
    }
    catalog.validated_role(LangItem::DropGlue)?;
    Ok(true)
}
