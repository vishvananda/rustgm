//! Reprove the provided methods admitted for the actual Iterator role.
use super::{Declaration, Declarations, DefId, DefKind, ItemId, Kind, LangItem, Report};
use crate::{
    sema::compiler_items::headers::Trait,
    types::{Primitive, TypeId, arguments::Argument, callable::Convention},
};

pub(in crate::sema) fn source_default(
    report: &Report,
    facts: &Declarations,
    member: DefId,
    owner: DefId,
) -> Result<Option<Declaration>, &'static str> {
    if !size_hint(report, member) {
        return Ok(None);
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source size_hint lost its actual provider catalog")?;
    let Some(iterator) = catalog
        .language(LangItem::Iterator)
        .filter(|role| role.definition == owner)
    else {
        return Ok(None);
    };
    let option = catalog
        .language(LangItem::Option)
        .ok_or("source size_hint requires its actual Option declaration")?;
    catalog.require_provider_report(report)?;
    let mut steps = 16;
    let checked = super::loop_headers::trait_header(report, facts, iterator.into(), &mut steps)
        .and_then(|header| require(facts, &header, ItemId(option.definition.0), member));
    crate::sema::header_attributes::charge_source_contract_query(facts, steps)
        .map_err(|_| "source size_hint exceeds its declaration work budget")?;
    checked?;
    Ok(Some(iterator))
}

pub(super) fn imported(
    report: &Report,
    facts: &Declarations,
    iterator: &Trait<'_>,
    option: ItemId,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(iterator.members.len());
    if let Some(&member) = iterator
        .members
        .iter()
        .find(|&&member| size_hint(report, member))
    {
        require(facts, iterator, option, member)?;
    }
    Ok(())
}

fn size_hint(report: &Report, member: DefId) -> bool {
    report
        .resolution
        .definitions
        .get(member.0 as usize)
        .is_some_and(|definition| {
            definition.kind == DefKind::Function
                && report.package.names.get(definition.name) == "size_hint"
        })
}

fn require(
    facts: &Declarations,
    iterator: &Trait<'_>,
    option: ItemId,
    member: DefId,
) -> Result<(), &'static str> {
    let function = facts
        .declaration_header(member)?
        .ok_or("source size_hint has no actual completed header")?;
    if function.body.is_none()
        || facts.associated.trait_owner(member) != Some(iterator.definition)
        || function.universals != 1
        || facts
            .generics
            .parameters
            .get(&member)
            .is_some_and(|own| !own.is_empty())
        || function.declaration.source().is_some() && facts.nonconst_trait_member(member)?
        || crate::sema::declaration_inputs::declared_own(
            &facts.generics,
            facts.crates.as_deref(),
            member,
        )?
        .len()
            != 0
    {
        return Err("source size_hint changed its actual provided conditional declaration");
    }
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
        bound_regions: 1,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source size_hint requires its actual safe Rust signature");
    };
    let Some([receiver]) = facts.types.list(parameters) else {
        return Err("source size_hint requires one actual input");
    };
    if !matches!(facts.types.kind(*receiver), Some(Kind::Reference { referent, region, mutable: false })
        if referent == iterator.self_type && region.bound_parts() == Some((0, 0)))
    {
        return Err("source size_hint requires its actual late-bound shared Self input");
    }
    let usize_ty = TypeId::primitive(Primitive::Usize);
    let Some([lower, upper]) = facts.types.kind(result).and_then(|kind| match kind {
        Kind::Tuple(fields) => facts.types.list(fields),
        _ => None,
    }) else {
        return Err("source size_hint requires its actual two-field result");
    };
    if *lower != usize_ty
        || !matches!(facts.types.kind(*upper), Some(Kind::Adt(owner, args))
            if owner == option && facts.types.arguments(args).is_some_and(|arguments| arguments.eq([Argument::Type(usize_ty)])))
    {
        return Err("source size_hint requires its actual (usize, Option<usize>) result");
    }
    Ok(())
}
