//! Complete source sizing headers precede every source-dependent selection answer.
use super::{
    Catalog,
    headers::{Flags, Trait},
};
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error, unit::FileId},
    sema::model::Declarations,
    support::bindings::LangItem,
    types::parameters::{clauses::Clause, predicates::Bound},
};

pub(in crate::sema) fn validate(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    if report.package.source_client().is_some() || report.package.staged_consumer().is_some() {
        return super::roles::admitted_headers::sizing(report, facts);
    }
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
    let header = Trait::source(report, facts, declaration).map_err(error)?;
    if header.attributes.requires_sizing_profile()
        && !report
            .resolution
            .source_bindings
            .as_deref()
            .is_some_and(|reserved| reserved.denies_implementation(declaration.definition))
    {
        return Err(error(
            "source sizing attributes require the actual deny-implementation contract",
        ));
    }
    if header.flags.unsafe_ {
        return Err(error(
            "source sizing profile requires a safe trait declaration",
        ));
    }
    if header
        .profile(
            &facts.types,
            Flags {
                unsafe_: false,
                constant: false,
                auto_: false,
            },
            0,
        )
        .is_err()
        || header.written.parameters != 0
        || !header.members.is_empty()
    {
        return Err(error(
            "source sizing trait header is outside the supported provider profile",
        ));
    }
    hierarchy(facts, catalog, kind, declaration.definition, &header).map_err(error)
}

pub(super) fn hierarchy(
    facts: &Declarations,
    catalog: &Catalog,
    kind: LangItem,
    definition: DefId,
    header: &Trait<'_>,
) -> Result<(), &'static str> {
    let expected = match kind {
        LangItem::Sized => Some(LangItem::MetaSized),
        LangItem::MetaSized => Some(LangItem::PointeeSized),
        LangItem::PointeeSized => None,
        _ => return Err("non-sizing language item in sizing validation"),
    };
    if header.written.supertraits != usize::from(expected.is_some()) {
        return Err("source sizing supertrait shape is outside the supported provider profile");
    }
    let expected = expected
        .map(|role| catalog.role(role))
        .transpose()?
        .flatten()
        .map(|declaration| declaration.definition);
    let mut found = false;
    for predicate in header.canonical_inputs(facts)?.iter() {
        let Clause::Predicate(predicate) = predicate.value else {
            return Err("source sizing profile does not support quantified header clauses");
        };
        let Bound::Trait(application) = predicate.bound else {
            return Err("source sizing profile requires only its actual trait hierarchy");
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or("source sizing header has an invalid trait application")?;
        if predicate.ty != header.self_type
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|a| a.len() != 0)
        {
            return Err("source sizing hierarchy must apply directly to its actual Self");
        }
        if application.item.0 == definition.0 {
            continue;
        }
        if Some(DefId(application.item.0)) != expected || found {
            return Err("source sizing profile requires its exact declared supertrait");
        }
        found = true;
    }
    if found != expected.is_some() {
        return Err("source sizing hierarchy is incomplete");
    }
    Ok(())
}
