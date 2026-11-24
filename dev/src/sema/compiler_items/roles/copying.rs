//! Copying roles validate declared contracts; no type gains a Copy permission here.
use super::super::headers::{Method, Trait, clone_method};
use super::{Catalog, Declaration, Declarations, Error, LangItem, Report, TypeId};
use crate::{
    resolve::model::{DefId, DefKind},
    sema::declaration_inputs::{self, Input},
    support::constness::Constness,
    types::{GenericArgs, Kind, callable::Convention, parameters::clauses::Clause},
};

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    role: LangItem,
    header: &Trait<'_>,
    complete: &mut super::RoleSet,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let expected = match role {
        LangItem::Copy => LangItem::Clone,
        LangItem::Clone => LangItem::Sized,
        LangItem::LegacyReceiver => LangItem::PointeeSized,
        _ => return Err(error("invalid copying role")),
    };
    require_header_super(facts, catalog, header, expected, Constness::Ordinary).map_err(error)?;
    let members = header.members;
    if role != LangItem::Clone {
        if !members.is_empty() {
            return Err(error("source copying marker must not declare members"));
        }
        return Ok(());
    }
    let method = catalog
        .language(LangItem::CloneFn)
        .ok_or_else(|| error("source Clone requires its actual clone_fn declaration"))?;
    let function = facts
        .declaration_header(method.definition)
        .map_err(error)?
        .ok_or_else(|| error("source clone_fn signature is not prepared"))?;
    if function.declaration != method.origin.into() {
        return Err(error("source clone_fn belongs to another declaration"));
    }
    clone_method(
        &facts.types,
        header,
        &Method {
            definition: method.definition,
            parent: method.parent,
            signature: function.signature,
            parameters: facts
                .generics
                .parameters
                .get(&method.definition)
                .map_or(0, Vec::len),
        },
    )
    .map_err(error)?;
    complete.insert(LangItem::CloneFn);
    Ok(())
}

pub(super) fn require_super(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    self_type: TypeId,
    expected: LangItem,
    constness: Constness,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let header = super::super::headers::Trait::source(report, facts, declaration).map_err(error)?;
    if header.self_type != self_type {
        return Err(error("source role requires its exact pinned supertrait"));
    }
    require_header_super(facts, catalog, &header, expected, constness).map_err(error)
}

pub(super) fn require_header_super(
    facts: &Declarations,
    catalog: &Catalog,
    header: &Trait<'_>,
    expected: LangItem,
    constness: Constness,
) -> Result<(), &'static str> {
    if header.written.supertraits != 1 {
        return Err("source role requires its exact pinned supertrait");
    }
    let expected = catalog
        .role(expected)?
        .ok_or("source role is missing its prerequisite declaration")?;
    let found = header.requires(
        &facts.types,
        expected.definition,
        catalog
            .role(LangItem::Sized)?
            .map(|declaration| declaration.definition),
        constness,
    );
    if !found {
        return Err("source role is missing its exact typed prerequisite");
    }
    Ok(())
}

/// This checked declaration carries a library safety contract, not a Clone implementation.
pub(super) fn validate_trivial(
    facts: &Declarations,
    catalog: &Catalog,
    header: &Trait<'_>,
) -> Result<(), &'static str> {
    if !header.members.is_empty()
        || !header
            .attributes
            .contains(crate::resolve::bindings::HeaderAttribute::Marker)
    {
        return Err("source TrivialClone requires its actual empty marker trait");
    }
    require_header_super(
        facts,
        catalog,
        header,
        LangItem::Clone,
        Constness::Conditional,
    )
}

pub(in crate::sema) fn source_clone_default(
    report: &Report,
    facts: &Declarations,
    member: DefId,
    owner: DefId,
) -> Result<Option<Declaration>, &'static str> {
    if !named_clone_from(report, member) {
        return Ok(None);
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source clone_from lost its actual provider catalog")?;
    let Some(clone) = catalog
        .header_role(LangItem::Clone)?
        .filter(|role| role.definition == owner)
    else {
        return Ok(None);
    };
    let header = Trait::source(report, facts, clone)?;
    require_clone_default(facts, catalog, &header, member)?;
    Ok(Some(clone))
}

pub(super) fn imported_clone_default(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    header: &Trait<'_>,
    steps: &mut usize,
) -> Result<(), &'static str> {
    if !header.flags.constant {
        return Ok(());
    }
    *steps = steps.saturating_add(header.members.len());
    let Some(&member) = header
        .members
        .iter()
        .find(|&&member| named_clone_from(report, member))
    else {
        return Ok(());
    };
    require_clone_default(facts, catalog, header, member)
}

fn named_clone_from(report: &Report, member: DefId) -> bool {
    report
        .resolution
        .definitions
        .get(member.0 as usize)
        .is_some_and(|definition| {
            definition.kind == DefKind::Function
                && report.package.names.get(definition.name) == "clone_from"
        })
}

fn require_clone_default(
    facts: &Declarations,
    catalog: &Catalog,
    owner: &Trait<'_>,
    member: DefId,
) -> Result<(), &'static str> {
    let function = facts
        .declaration_header(member)?
        .ok_or("source clone_from has no actual completed header")?;
    if !owner.flags.constant
        || facts.associated.trait_owner(member) != Some(owner.definition)
        || function.body.is_none()
        || function.universals != 2
        || facts
            .generics
            .parameters
            .get(&member)
            .is_some_and(|parameters| !parameters.is_empty())
        || function.declaration.source().is_some() && facts.nonconst_trait_member(member)?
    {
        return Err("source clone_from changed its actual provided conditional declaration");
    }
    let Some(Kind::Function {
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: Convention::RUST,
        bound_regions: 2,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source clone_from requires its exact safe Rust signature");
    };
    let Some([receiver, incoming]) = facts.types.list(parameters) else {
        return Err("source clone_from requires its actual receiver and source");
    };
    for (slot, ty, mutable) in [(0, *receiver, true), (1, *incoming, false)] {
        if !matches!(facts.types.kind(ty), Some(Kind::Reference { referent, mutable: actual, region })
            if referent == owner.self_type && actual == mutable && region.bound_parts() == Some((0, slot)))
        {
            return Err("source clone_from changed its distinct late Self borrows");
        }
    }
    require_clone_destruct(facts, catalog, owner.self_type, member)
}

fn require_clone_destruct(
    facts: &Declarations,
    catalog: &Catalog,
    subject: TypeId,
    member: DefId,
) -> Result<(), &'static str> {
    const BOUND: &str = "source clone_from requires its actual conditional Self: Destruct premise";
    let destruct = catalog.role(LangItem::Destruct)?.ok_or(BOUND)?;
    let inputs =
        declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), member)?;
    let Some(Input {
        value: Clause::Predicate(predicate),
        ..
    }) = inputs.get(0).filter(|_| inputs.len() == 1)
    else {
        return Err(BOUND);
    };
    let Some((application, Constness::Conditional)) = predicate.bound.trait_parts() else {
        return Err(BOUND);
    };
    if predicate.ty != subject
        || !facts
            .types
            .trait_application(application)
            .is_some_and(|application| {
                application.item.0 == destruct.definition.0
                    && application.arguments == GenericArgs::EMPTY
            })
    {
        return Err(BOUND);
    }
    Ok(())
}
