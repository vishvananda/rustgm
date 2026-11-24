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
    Poll,
    CoroutineState,
}
impl Group {
    pub(super) const SOURCE: [Self; 3] = [Self::Option, Self::Result, Self::Ordering];

    pub(super) const ALL: [Self; 5] = [
        Self::Option,
        Self::Result,
        Self::Ordering,
        Self::Poll,
        Self::CoroutineState,
    ];

    pub(super) const fn roles(self) -> &'static [LangItem] {
        match self {
            Self::Option => &[LangItem::Option, LangItem::OptionSome, LangItem::OptionNone],
            Self::Result => &[LangItem::ResultOk, LangItem::ResultErr],
            Self::Ordering => &[LangItem::Ordering],
            Self::Poll => &[LangItem::Poll, LangItem::PollReady, LangItem::PollPending],
            Self::CoroutineState => &[LangItem::CoroutineState],
        }
    }

    pub(super) fn check(
        self,
        facts: &Declarations,
        catalog: &Catalog,
    ) -> Result<bool, &'static str> {
        match self {
            Self::Option | Self::Poll => return unary_enum(facts, catalog, self.roles()),
            Self::CoroutineState => return coroutine_state(facts, catalog),
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

fn unary_enum(
    facts: &Declarations,
    catalog: &Catalog,
    roles: &[LangItem],
) -> Result<bool, &'static str> {
    let &[owner, payload, empty] = roles else {
        return Err("source enum requires all three roles");
    };
    let (owner, payload, empty) = (
        catalog.role(owner)?,
        catalog.role(payload)?,
        catalog.role(empty)?,
    );
    let (Some(owner), Some(payload), Some(empty)) = (owner, payload, empty) else {
        return absent([owner, payload, empty]);
    };
    let owner = ItemId(owner.definition.0);
    let [formal] = formals(facts, owner, 2)? else {
        return Err("source enum requires its actual single type formal");
    };
    let Argument::Type(ty) = formal.argument else {
        return Err("source enum requires its actual type payload");
    };
    check_variant(facts, owner, payload, Some(ty))?;
    check_variant(facts, owner, empty, None)?;
    Ok(true)
}

fn coroutine_state(facts: &Declarations, catalog: &Catalog) -> Result<bool, &'static str> {
    let Some(role) = catalog.role(LangItem::CoroutineState)? else {
        return Ok(false);
    };
    let owner = ItemId(role.definition.0);
    let [yielded, returned] = formals(facts, owner, 2)? else {
        return Err("CoroutineState requires its actual two type formals");
    };
    for (&variant, formal) in facts
        .members
        .variants(owner)
        .iter()
        .zip([yielded, returned])
    {
        let Argument::Type(ty) = formal.argument else {
            return Err("CoroutineState requires distinct actual type payloads");
        };
        let record = facts
            .members
            .variants
            .get(variant.0 as usize)
            .ok_or("CoroutineState lost its actual variant declaration")?;
        if record.owner != owner || variant_payload(facts, variant, record)? != Some(ty) {
            return Err("CoroutineState changed an actual variant or its payload type");
        }
    }
    Ok(true)
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
    if variant_payload(facts, id, variant)? != payload {
        return Err("source enum variant role changed its actual payload type");
    }
    Ok(())
}

fn variant_payload(
    facts: &Declarations,
    id: VariantId,
    variant: &crate::sema::members::Variant,
) -> Result<Option<TypeId>, &'static str> {
    let fields = facts
        .members
        .fields(id)
        .ok_or("source enum variant lost its actual fields")?;
    match (variant.shape, fields) {
        (Shape::Unit, []) => Ok(None),
        (Shape::Tuple, [field]) => Ok(Some(field.ty)),
        _ => Err("source enum variant role requires its exact unit or single-field shape"),
    }
}
