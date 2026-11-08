//! Actual source `PhantomData` contributes logical ownership, never invented storage.
use crate::{
    resolve::{
        bindings::Declaration,
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
    },
    sema::model::Declarations,
    support::{bindings::LangItem, constness::Constness},
    syntax::{
        ast::Kind as Syntax,
        item::{ItemKind, Shape},
    },
    types::{ItemId, Kind, TypeId, aggregate, arguments::Argument},
};

pub(in crate::sema) fn declaration(
    report: &Report,
    facts: &Declarations,
    item: ItemId,
) -> Result<bool, Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some(declaration) = catalog.language(LangItem::PhantomData) else {
        return Ok(false);
    };
    if declaration.definition.0 != item.0 {
        return Ok(false);
    }
    crate::sema::header_attributes::charge_source_contract_query(facts, 1)?;
    let error = |message| Declarations::error(report, declaration.origin, message);
    catalog.require_provider_report(report).map_err(error)?;
    let actual = report
        .resolution
        .definitions
        .get(item.0 as usize)
        .ok_or_else(|| error("source PhantomData lost its actual definition"))?;
    let ast = &report.package.unit(declaration.origin.file).configured.ast;
    let Syntax::Item(index) = ast.node(declaration.origin.node).kind else {
        return Err(error(
            "source PhantomData requires its actual unit structure",
        ));
    };
    let written = ast.items[index as usize];
    if declaration.parent.is_some()
        || actual.kind != DefKind::Struct
        || actual.node != Some(declaration.origin)
        || !matches!(written.kind, ItemKind::Struct { shape: Shape::Unit, fields } if ast.list(fields).is_empty())
        || ast.list(written.generics).len() != 1
    {
        return Err(error(
            "source PhantomData requires its actual module-owned unit structure",
        ));
    }
    let target = formal(facts, declaration.definition).map_err(error)?;
    let pointee = catalog
        .header_role(LangItem::PointeeSized)
        .map_err(error)?
        .ok_or_else(|| error("source PhantomData requires actual PointeeSized"))?;
    let mut inputs = 0;
    let bounded = pointee_bounded(
        facts,
        declaration.definition,
        target,
        pointee.definition,
        &mut inputs,
    )
    .map_err(error)?;
    crate::sema::header_attributes::charge_source_contract_query(facts, inputs)?;
    if !bounded {
        return Err(error(
            "source PhantomData requires its actual PointeeSized formal bound",
        ));
    }
    Ok(true)
}

/// Whether `definition` declares the ordinary, argument-free bound
/// `target: PointeeSized`, as written in source or carried by its import.
/// Each declared input read is charged to `steps`.
fn pointee_bounded(
    facts: &Declarations,
    definition: DefId,
    target: TypeId,
    pointee: DefId,
    steps: &mut usize,
) -> Result<bool, &'static str> {
    Ok(crate::sema::declaration_inputs::declared(
        &facts.generics,
        facts.crates.as_deref(),
        definition,
    )?
    .any(|input| {
        *steps = steps.saturating_add(1);
        let crate::types::parameters::clauses::Clause::Predicate(predicate) = input.value else {
            return false;
        };
        predicate.ty == target
            && predicate.bound.trait_parts().is_some_and(|(id, effect)| {
                effect == Constness::Ordinary
                    && facts
                        .types
                        .trait_application(id)
                        .is_some_and(|application| {
                            application.item.0 == pointee.0
                                && facts
                                    .types
                                    .arguments(application.arguments)
                                    .is_some_and(|arguments| arguments.len() == 0)
                        })
            })
    }))
}

/// The declaration's one type formal, with no default.
fn formal(facts: &Declarations, definition: DefId) -> Result<TypeId, &'static str> {
    let item = ItemId(definition.0);
    let parameters = facts
        .generics
        .parameters
        .get(&definition)
        .ok_or("source PhantomData parameters are not complete")?;
    let [parameter] = parameters.as_slice() else {
        return Err("source PhantomData requires one actual type formal");
    };
    let Argument::Type(target) = parameter.argument else {
        return Err("source PhantomData requires a type formal");
    };
    if parameter.default.is_some()
        || parameter.anonymous
        || !matches!(facts.types.kind(target), Some(Kind::Parameter(id)) if facts.types.parameter(id).is_some_and(|p| p.owner == item && p.index == 0))
    {
        return Err("source PhantomData formal changed its actual owner or default");
    }
    Ok(target)
}

/// A source client re-proves its provider's `PhantomData` over its own
/// interned facts: the provider's checks, and the logical category it staged
/// for the struct the role names. The consumer's variance pass re-derives
/// every other imported formal, so only this struct may leave one unused.
pub(super) fn imported(
    facts: &Declarations,
    catalog: &super::Catalog,
    declaration: super::Role,
    steps: &mut usize,
) -> Result<(), &'static str> {
    let definition = declaration.definition;
    let item = ItemId(definition.0);
    let variants = facts.members.variants(item);
    *steps = steps.saturating_add(4 + variants.len());
    let unit = matches!(variants, [variant] if facts.members.variants[variant.0 as usize].shape == Shape::Unit
        && facts.members.fields(*variant).is_some_and(<[_]>::is_empty));
    if declaration.parent.is_some()
        || !unit
        || facts
            .aggregates
            .get(item)
            .is_none_or(|actual| actual.kind != aggregate::Kind::PhantomData)
    {
        return Err("imported PhantomData requires its module-owned unit structure");
    }
    let target = formal(facts, definition)?;
    let pointee = catalog
        .role(LangItem::PointeeSized)?
        .ok_or("imported PhantomData requires actual PointeeSized")?;
    if !pointee_bounded(facts, definition, target, pointee.definition, steps)? {
        return Err("imported PhantomData requires its actual PointeeSized formal bound");
    }
    let ty = facts
        .definitions
        .get(item.0 as usize)
        .copied()
        .flatten()
        .ok_or("imported PhantomData has no canonical actual type")?;
    facts
        .aggregates
        .phantom_argument(&facts.types, ty)?
        .ok_or("imported PhantomData lost its logical type argument")?;
    Ok(())
}

/// Nominal role completion follows the existing deferred bound/field worklist,
/// independently of the earlier callable-role header completion.
pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(claimed) = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::PhantomData))
    else {
        return Ok(());
    };
    require_completed(report, facts, claimed)?;
    if super::roles::nominal_completed(report, facts, LangItem::PhantomData, claimed)? {
        return Ok(());
    }
    super::roles::complete_nominal(facts, LangItem::PhantomData, claimed)
}

pub(super) fn require_completed(
    report: &Report,
    facts: &Declarations,
    claimed: Declaration,
) -> Result<(), Error> {
    let item = ItemId(claimed.definition.0);
    let error = |message| Declarations::error(report, claimed.origin, message);
    if !declaration(report, facts, item)?
        || facts
            .aggregates
            .get(item)
            .is_none_or(|definition| definition.kind != aggregate::Kind::PhantomData)
    {
        return Err(error(
            "source PhantomData requires its completed checked nominal category",
        ));
    }
    let ty = facts
        .definitions
        .get(item.0 as usize)
        .copied()
        .flatten()
        .ok_or_else(|| error("source PhantomData has no canonical actual type"))?;
    facts
        .aggregates
        .phantom_argument(&facts.types, ty)
        .map_err(error)?
        .ok_or_else(|| error("source PhantomData lost its logical type argument"))?;
    Ok(())
}

#[cfg(test)]
#[path = "../../../tests/invariants/phantom_ownership.rs"]
mod tests;
