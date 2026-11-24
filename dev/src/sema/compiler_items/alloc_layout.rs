//! The pinned allocation layout retains its ordinary typed fields and alignment values.
use super::{Declaration, LangItem, Report, Role};
use crate::{
    sema::{header_attributes, model::Declarations},
    types::{GenericArgs, ItemId, Kind, Primitive, TypeId, aggregate},
};

// The selected source profile targets x86-64. AlignmentEnum has one value
// for each power of two representable in the target's usize.
const ALIGNMENTS: usize = u64::BITS as usize;
const WORK: usize = ALIGNMENTS + 12;

pub(in crate::sema) fn source_role(
    report: &Report,
    facts: &Declarations,
    enabled: bool,
) -> Result<Option<Declaration>, &'static str> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("allocation layout lost its role catalog")?;
    let Some(declaration) = catalog.language(LangItem::AllocLayout) else {
        return Ok(None);
    };
    if !enabled {
        return Err("allocation layout transport requires version fourteen");
    }
    catalog.require_provider_report(report)?;
    header_attributes::charge_source_contract_query(facts, WORK)
        .map_err(|_| "allocation layout declaration query exceeds its source work budget")?;
    require_shape(facts, declaration.into())?;
    Ok(Some(declaration))
}

pub(super) fn imported(
    facts: &Declarations,
    declaration: Role,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(WORK);
    require_shape(facts, declaration)
}

fn require_shape(facts: &Declarations, layout: Role) -> Result<(), &'static str> {
    let definition = layout.definition;
    let item = ItemId(definition.0);
    if layout.parent.is_some()
        || !facts.generics.nominal_contracts.contains(&definition)
        || facts
            .generics
            .parameters
            .get(&definition)
            .is_some_and(|formals| !formals.is_empty())
    {
        return Err("allocation layout requires its actual closed module declaration");
    }
    let [size, alignment] = fields(facts, item, aggregate::Repr::default())? else {
        return Err("allocation layout requires its size and alignment fields");
    };
    if *size != TypeId::primitive(Primitive::Usize) {
        return Err("allocation layout changed its target usize size field");
    }
    let transparent = aggregate::Repr {
        representation: aggregate::Representation::Transparent,
        ..aggregate::Repr::default()
    };
    let [enumeration] = fields(facts, closed_nominal(facts, *alignment)?, transparent)? else {
        return Err("allocation layout requires its transparent alignment field");
    };
    alignment_values(facts, closed_nominal(facts, *enumeration)?)
}

fn closed_nominal(facts: &Declarations, ty: TypeId) -> Result<ItemId, &'static str> {
    match facts.types.kind(ty) {
        Some(Kind::Adt(owner, GenericArgs::EMPTY)) => Ok(owner),
        _ => Err("allocation layout alignment requires its actual closed nominal field"),
    }
}

fn fields(
    facts: &Declarations,
    item: ItemId,
    repr: aggregate::Repr,
) -> Result<&[TypeId], &'static str> {
    let owner = facts
        .aggregates
        .get(item)
        .filter(|owner| owner.kind == aggregate::Kind::Struct && owner.repr == repr)
        .ok_or("allocation layout or alignment changed its actual struct representation")?;
    let [variant] = facts.aggregates.variants(owner) else {
        return Err("allocation layout lost its actual structural fields");
    };
    facts
        .types
        .list(variant.fields)
        .ok_or("allocation layout has invalid checked fields")
}

fn alignment_values(facts: &Declarations, item: ItemId) -> Result<(), &'static str> {
    let repr = aggregate::Repr {
        tag: Some(Primitive::Usize),
        ..aggregate::Repr::default()
    };
    let owner = facts
        .aggregates
        .get(item)
        .filter(|owner| owner.kind == aggregate::Kind::Enum && owner.repr == repr)
        .ok_or("allocation layout alignment requires its actual repr(usize) enum")?;
    let variants = facts.aggregates.variants(owner);
    if !facts.aggregates.discriminants_ready(item)
        || variants.len() != ALIGNMENTS
        || variants.iter().enumerate().any(|(bit, variant)| {
            variant.discriminant != 1_u128 << bit || facts.types.list(variant.fields) != Some(&[])
        })
    {
        return Err("allocation layout changed its target powers-of-two alignment values");
    }
    Ok(())
}
