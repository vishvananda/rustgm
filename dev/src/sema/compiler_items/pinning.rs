//! Pin construction retains the real inherent source body and ordinary call checks.
#[cfg(test)]
use super::headers::functions::declaration_signature;
use crate::{
    resolve::{expansion::Report, model::DefId, package::Error},
    sema::{associated::OwnerKind, declaration_inputs, model::Declarations},
    support::{bindings::LangItem, constness::Constness},
    types::{
        GenericArgs, Kind, TypeId, arguments::Argument, callable::Convention,
        parameters::predicates::Bound,
    },
};
#[cfg(test)]
#[path = "../../../tests/invariants/pin_headers.rs"]
mod tests;

pub(in crate::sema) fn complete(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(declaration) =
        super::headers::functions::inherent(report, facts, LangItem::PinNewUnchecked)?
    else {
        return Ok(());
    };
    super::roles::complete_constructor(facts, LangItem::PinNewUnchecked, declaration)
}

pub(super) fn imported(
    facts: &Declarations,
    role: super::Role,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(20);
    let (owner, origin) = checked_constructor(facts, role.definition)?;
    if origin != role.origin || origin.source().is_some() || role.parent != Some(owner) {
        return Err("imported Pin constructor changed its actual provider member identity");
    }
    Ok(())
}

pub(in crate::sema) fn checked_constructor(
    facts: &Declarations,
    definition: DefId,
) -> Result<(DefId, crate::resolve::body_origin::DeclarationOrigin), &'static str> {
    let owner = facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id))
        .filter(|owner| {
            matches!(
                owner.kind,
                OwnerKind::Impl {
                    trait_type: None,
                    ..
                } | OwnerKind::ImportedInherent(_)
            )
        })
        .ok_or("Pin constructor requires its actual inherent owner")?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("Pin constructor requires its actual source catalog")?;
    let pin = catalog
        .role(LangItem::Pin)?
        .ok_or("Pin constructor requires its actual Pin declaration")?;
    let deref = catalog
        .role(LangItem::Deref)?
        .ok_or("Pin constructor requires its actual Deref declaration")?;
    let header = facts
        .declaration_header(definition)?
        .ok_or("Pin constructor lost its actual function header")?;
    let argument = constructor_signature(
        facts,
        definition,
        owner.self_definition,
        pin.definition,
        &header,
    )?;
    let meta = catalog
        .role(LangItem::MetaSized)?
        .ok_or("Pin constructor requires its actual implicit sizing declaration")?;
    let origins = declaration_inputs::declared_own(
        &facts.generics,
        facts.crates.as_deref(),
        owner.self_definition,
    )?;
    if origins.len() > 3
        || !constructor_bounds(facts, origins, argument, deref.definition, meta.definition)?
        || declaration_inputs::declared_own(&facts.generics, facts.crates.as_deref(), definition)?
            .len()
            != 0
    {
        return Err("Pin constructor changed its actual Ptr: Deref condition");
    }
    Ok((owner.self_definition, header.declaration))
}

fn constructor_bounds(
    facts: &Declarations,
    origins: declaration_inputs::Inputs<'_>,
    argument: TypeId,
    deref: DefId,
    meta: DefId,
) -> Result<bool, &'static str> {
    let mut found = false;
    for input in origins.iter() {
        let predicate = input.value.ordinary()?;
        if predicate.ty != argument {
            return Ok(false);
        }
        if predicate.bound == Bound::Sized {
            continue;
        }
        let Some((application, Constness::Ordinary)) = predicate.bound.trait_parts() else {
            return Ok(false);
        };
        let application = facts
            .types
            .trait_application(application)
            .ok_or("invalid Pin bound")?;
        if application.arguments != GenericArgs::EMPTY
            || ![deref.0, meta.0].contains(&application.item.0)
        {
            return Ok(false);
        }
        found |= application.item.0 == deref.0;
    }
    Ok(found)
}

fn constructor_signature(
    facts: &Declarations,
    definition: DefId,
    owner: DefId,
    pin: DefId,
    header: &crate::sema::function_headers::DeclarationHeader,
) -> Result<TypeId, &'static str> {
    let Some(Kind::Function {
        bound_regions: 0,
        unsafe_: true,
        convention: Convention::RUST,
        parameters,
        result,
    }) = facts.types.kind(header.signature)
    else {
        return Err("Pin constructor requires its actual unsafe Rust signature");
    };
    let Some([parameter]) = facts.types.list(parameters) else {
        return Err("Pin constructor requires the actual pointer argument");
    };
    let supplied = facts
        .generics
        .parameters
        .get(&owner)
        .map_or(&[][..], Vec::as_slice);
    let declared = [Argument::Type(*parameter)];
    if !facts.is_const_function(definition)
        || header.body.is_none()
        || supplied
            .iter()
            .map(|parameter| parameter.argument)
            .ne(declared)
        || facts
            .generics
            .parameters
            .get(&definition)
            .is_some_and(|parameters| !parameters.is_empty())
        || facts
            .associated
            .schema(owner)
            .is_none_or(|schema| schema.self_type != result)
        || !matches!(facts.types.kind(result), Some(Kind::Adt(item, arguments))
            if item.0 == pin.0 && facts.types.arguments(arguments).is_some_and(|args| args.eq(declared)))
    {
        return Err("Pin constructor requires its actual generic Ptr and Pin<Ptr> result");
    }
    Ok(*parameter)
}
