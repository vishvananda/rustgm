//! Bind real enum declarations and constructors without changing their layout.
use super::{Catalog, Role};
use crate::{
    resolve::{body_origin::DeclarationOrigin, model::DefId},
    sema::{
        members::{Origin, VariantId},
        model::Declarations,
    },
    support::bindings::LangItem,
    syntax::item::Shape,
    types::{ItemId, TypeId, aggregate, arguments::Argument},
};

#[derive(Clone, Copy)]
pub(super) enum Group {
    Option,
    Result,
    Ordering,
}
impl Group {
    pub(super) const ALL: [Self; 3] = [Self::Option, Self::Result, Self::Ordering];

    pub(super) const fn roles(self) -> &'static [LangItem] {
        match self {
            Self::Option => &[LangItem::Option, LangItem::OptionSome, LangItem::OptionNone],
            Self::Result => &[LangItem::ResultOk, LangItem::ResultErr],
            Self::Ordering => &[LangItem::Ordering],
        }
    }

    pub(super) fn check(
        self,
        facts: &Declarations,
        catalog: &Catalog,
    ) -> Result<bool, &'static str> {
        match self {
            Self::Option => {
                let owner = catalog.role(LangItem::Option)?;
                let some = catalog.role(LangItem::OptionSome)?;
                let none = catalog.role(LangItem::OptionNone)?;
                let (Some(owner), Some(some), Some(none)) = (owner, some, none) else {
                    return absent([owner, some, none]);
                };
                let owner = ItemId(owner.definition.0);
                let [payload] = formals(facts, owner, 2)? else {
                    return Err("Option requires its actual single type formal");
                };
                let Argument::Type(payload) = payload.argument else {
                    return Err("Option requires its actual type payload");
                };
                check_variant(facts, owner, some, Some(payload))?;
                check_variant(facts, owner, none, None)?;
            }
            Self::Result => {
                let ok = catalog.role(LangItem::ResultOk)?;
                let err = catalog.role(LangItem::ResultErr)?;
                let (Some(ok), Some(err)) = (ok, err) else {
                    return absent([ok, err]);
                };
                let (_, value_variant) = variant(facts, ok)?;
                let owner = value_variant.owner;
                let [value, error] = formals(facts, owner, 2)? else {
                    return Err("Result requires its actual two type formals");
                };
                let (Argument::Type(value), Argument::Type(error)) =
                    (value.argument, error.argument)
                else {
                    return Err("Result requires its actual type payloads");
                };
                check_variant(facts, owner, ok, Some(value))?;
                check_variant(facts, owner, err, Some(error))?;
            }
            Self::Ordering => {
                let Some(declaration) = catalog.role(LangItem::Ordering)? else {
                    return Ok(false);
                };
                let owner = ItemId(declaration.definition.0);
                if facts
                    .generics
                    .parameters
                    .get(&declaration.definition)
                    .is_some_and(|parameters| !parameters.is_empty())
                {
                    return Err("Ordering must have its actual parameterless declaration");
                }
                super::compare_intrinsic::ordering_layout(facts, owner)?;
            }
        }
        Ok(true)
    }
}

fn absent<const N: usize>(roles: [Option<Role>; N]) -> Result<bool, &'static str> {
    if roles.iter().any(Option::is_some) {
        return Err("source enum roles require their complete actual declaration group");
    }
    Ok(false)
}

fn formals(
    facts: &Declarations,
    owner: ItemId,
    variants: usize,
) -> Result<&[crate::sema::generics::Parameter], &'static str> {
    if facts
        .aggregates
        .get(owner)
        .is_none_or(|definition| definition.kind != aggregate::Kind::Enum)
        || facts.members.variants(owner).len() != variants
    {
        return Err("source enum role changed its actual nominal shape");
    }
    Ok(facts
        .generics
        .parameters
        .get(&DefId(owner.0))
        .map_or(&[], Vec::as_slice))
}

fn variant(
    facts: &Declarations,
    role: Role,
) -> Result<(VariantId, &crate::sema::members::Variant), &'static str> {
    let id = facts
        .members
        .constructor(role.definition)
        .ok_or("source enum role requires its actual constructor")?;
    let variant = facts
        .members
        .variants
        .get(id.0 as usize)
        .ok_or("source enum variant role lost its actual member")?;
    Ok((id, variant))
}

fn check_variant(
    facts: &Declarations,
    owner: ItemId,
    role: Role,
    payload: Option<TypeId>,
) -> Result<(), &'static str> {
    let (id, variant) = variant(facts, role)?;
    let origin = match role.origin {
        DeclarationOrigin::Source(origin) => Origin::Source(origin),
        DeclarationOrigin::Imported(_) => Origin::External(DefId(owner.0)),
    };
    if variant.owner != owner
        || variant.origin != origin
        || facts.members.variants(owner).get(variant.ordinal as usize) != Some(&id)
    {
        return Err("source enum variant role changed its actual owner or identity");
    }
    let fields = facts
        .members
        .fields(id)
        .ok_or("source enum variant lost its actual fields")?;
    let present = match (variant.shape, fields) {
        (Shape::Unit, []) => None,
        (Shape::Tuple, [field]) => Some(field.ty),
        _ => return Err("source enum variant role requires its exact unit or single-field shape"),
    };
    if present != payload {
        return Err("source enum variant role changed its actual payload type");
    }
    Ok(())
}
