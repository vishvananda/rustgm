//! Actual source PhantomData contributes logical ownership, never invented storage.
use crate::{
    resolve::{bindings::Declaration, expansion::Report, model::DefKind, package::Error},
    sema::model::Declarations,
    support::{bindings::LangItem, constness::Constness},
    syntax::{
        ast::Kind as Syntax,
        item::{ItemKind, Shape},
    },
    types::{ItemId, Kind, aggregate, arguments::Argument},
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
    let target = formal(report, facts, declaration)?;
    let pointee = catalog
        .header_role(LangItem::PointeeSized)
        .map_err(error)?
        .ok_or_else(|| error("source PhantomData requires actual PointeeSized"))?;
    let predicates = facts.generics.predicates.get(&declaration.definition);
    crate::sema::header_attributes::charge_source_contract_query(
        facts,
        predicates.map_or(0, Vec::len),
    )?;
    if !facts
        .generics
        .declared_predicates_in_scope(declaration.definition)
        .any(|clause| {
            let crate::types::parameters::clauses::Clause::Predicate(predicate) = clause.value
            else {
                return false;
            };
            predicate.ty == target
                && predicate.bound.trait_parts().is_some_and(|(id, effect)| {
                    effect == Constness::Ordinary
                        && facts
                            .types
                            .trait_application(id)
                            .is_some_and(|application| {
                                application.item.0 == pointee.definition.0
                                    && facts
                                        .types
                                        .arguments(application.arguments)
                                        .is_some_and(|arguments| arguments.len() == 0)
                            })
                })
        })
    {
        return Err(error(
            "source PhantomData requires its actual PointeeSized formal bound",
        ));
    }
    Ok(true)
}

fn formal(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
) -> Result<crate::types::TypeId, Error> {
    let item = ItemId(declaration.definition.0);
    let error = |message| Declarations::error(report, declaration.origin, message);
    let parameters = facts
        .generics
        .parameters
        .get(&declaration.definition)
        .ok_or_else(|| error("source PhantomData parameters are not complete"))?;
    let [parameter] = parameters.as_slice() else {
        return Err(error("source PhantomData requires one actual type formal"));
    };
    let Argument::Type(target) = parameter.argument else {
        return Err(error("source PhantomData requires a type formal"));
    };
    if parameter.default.is_some()
        || parameter.anonymous
        || !matches!(facts.types.kind(target), Some(Kind::Parameter(id)) if facts.types.parameter(id).is_some_and(|p| p.owner == item && p.index == 0))
    {
        return Err(error(
            "source PhantomData formal changed its actual owner or default",
        ));
    }
    Ok(target)
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
