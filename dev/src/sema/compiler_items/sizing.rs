//! Complete source sizing headers precede every source-dependent selection answer.
use super::Catalog;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::FileId,
    },
    sema::{associated::OwnerKind, model::Declarations},
    support::bindings::LangItem,
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{
        Kind,
        parameters::{clauses::Clause, predicates::Bound},
    },
};

pub(in crate::sema) fn validate(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    if catalog.sizing.is_some() {
        return Err(Error::at(
            FileId(0),
            0,
            "source sizing declarations were validated twice",
        ));
    }
    if report
        .resolution
        .source_bindings
        .as_deref()
        .is_none_or(|r| r.provider != catalog.provider)
    {
        return Err(Error::at(
            FileId(0),
            0,
            "source sizing provider changed before validation",
        ));
    }
    let mut definitions = Vec::new();
    for kind in [LangItem::Sized, LangItem::MetaSized, LangItem::PointeeSized] {
        let declaration = catalog.language(kind).ok_or_else(|| {
            Error::at(
                FileId(0),
                0,
                "source provider requires distinct Sized, MetaSized and PointeeSized declarations",
            )
        })?;
        validate_header(report, facts, catalog, kind, declaration)?;
        definitions.push(declaration.definition);
    }
    let definitions: [DefId; 3] = definitions
        .try_into()
        .map_err(|_| Error::at(FileId(0), 0, "source sizing declaration count changed"))?;
    if definitions[0] == definitions[1]
        || definitions[0] == definitions[2]
        || definitions[1] == definitions[2]
    {
        return Err(Error::at(
            FileId(0),
            0,
            "source sizing identities must be distinct",
        ));
    }
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| Error::at(FileId(0), 0, "source sizing catalog disappeared"))?
        .sizing = Some(definitions);
    Ok(())
}

fn validate_header(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    kind: LangItem,
    declaration: crate::resolve::bindings::Declaration,
) -> Result<(), Error> {
    let key = declaration.origin;
    let error = |message| Declarations::error(report, key, message);
    let definition = report
        .resolution
        .definitions
        .get(declaration.definition.0 as usize)
        .ok_or_else(|| error("source sizing identity has no declaration"))?;
    let unit = report.package.unit(key.file);
    let Syntax::Item(index) = unit.configured.ast.node(key.node).kind else {
        return Err(error("source sizing identity has no item owner"));
    };
    let item = unit.configured.ast.items[index as usize];
    let ItemKind::Trait {
        unsafe_: false,
        bounds,
        ..
    } = item.kind
    else {
        return Err(error(
            "source sizing profile requires a safe trait declaration",
        ));
    };
    let owner = facts
        .associated
        .owner(declaration.definition)
        .and_then(|id| facts.associated.get(id))
        .ok_or_else(|| error("source sizing trait is not prepared"))?;
    let owner_id = facts
        .associated
        .owner(declaration.definition)
        .ok_or_else(|| error("source sizing trait lost its owner"))?;
    let self_type = facts
        .associated
        .schema(declaration.definition)
        .ok_or_else(|| error("source sizing trait has no completed Self schema"))?
        .self_type;
    if definition.kind != DefKind::Trait
        || definition.node != Some(key)
        || declaration.parent.is_some()
        || !matches!(owner.kind, OwnerKind::Trait { definition, unsafe_: false, constant: false, auto_: false } if definition == declaration.definition)
        || !matches!(facts.types.kind(self_type), Some(Kind::Parameter(_)))
        || item.generics.start != item.generics.end
        || facts
            .generics
            .parameters
            .get(&declaration.definition)
            .is_none_or(|p| !p.is_empty())
        || facts
            .associated
            .owner_members(owner_id)
            .is_none_or(|m| !m.is_empty())
    {
        return Err(error(
            "source sizing trait header is outside the supported provider profile",
        ));
    }
    validate_hierarchy(report, facts, catalog, kind, declaration, self_type, bounds)
}

fn validate_hierarchy(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    kind: LangItem,
    declaration: crate::resolve::bindings::Declaration,
    self_type: crate::types::TypeId,
    bounds: crate::syntax::ast::List,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let unit = report.package.unit(declaration.origin.file);
    let expected = match kind {
        LangItem::Sized => Some(LangItem::MetaSized),
        LangItem::MetaSized => Some(LangItem::PointeeSized),
        LangItem::PointeeSized => None,
        _ => {
            return Err(error("non-sizing language item in sizing validation"));
        }
    };
    if unit.configured.ast.list(bounds).len() != usize::from(expected.is_some()) {
        return Err(error(
            "source sizing supertrait shape is outside the supported provider profile",
        ));
    }
    let expected = expected
        .and_then(|kind| catalog.language(kind))
        .map(|d| d.definition);
    let mut found = false;
    for predicate in facts
        .generics
        .predicates
        .get(&declaration.definition)
        .into_iter()
        .flatten()
    {
        let Clause::Predicate(predicate) = predicate.value else {
            return Err(error(
                "source sizing profile does not support quantified header clauses",
            ));
        };
        let Bound::Trait(application) = predicate.bound else {
            return Err(error(
                "source sizing profile requires only its actual trait hierarchy",
            ));
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or_else(|| error("source sizing header has an invalid trait application"))?;
        if predicate.ty != self_type
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err(error(
                "source sizing hierarchy must apply directly to its actual Self",
            ));
        }
        if application.item.0 == declaration.definition.0 {
            continue;
        }
        if Some(DefId(application.item.0)) != expected || found {
            return Err(error(
                "source sizing profile requires its exact declared supertrait",
            ));
        }
        found = true;
    }
    if found != expected.is_some() {
        return Err(error("source sizing hierarchy is incomplete"));
    }
    Ok(())
}
