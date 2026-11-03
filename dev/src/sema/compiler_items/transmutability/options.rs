//! Four actual named boolean fields retain their source owner and canonical value.
use crate::{
    resolve::bindings::Declaration,
    sema::{members::Name, model::Declarations},
    support::bindings::LangItem,
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId, aggregate,
        arguments::{ConstId, ConstKind, trees::Aggregate},
    },
};
const FIELDS: [&str; 4] = ["alignment", "lifetimes", "safety", "validity"];

pub(super) fn slots(
    facts: &Declarations,
    declaration: Declaration,
) -> Result<[usize; 4], &'static str> {
    let item = ItemId(declaration.definition.0);
    let nominal = facts
        .aggregates
        .get(item)
        .ok_or("transmute_opts requires its actual nominal fields")?;
    if declaration.parent.is_some()
        || nominal.kind != aggregate::Kind::Struct
        || facts
            .generics
            .parameters
            .get(&declaration.definition)
            .is_some_and(|p| !p.is_empty())
    {
        return Err("transmute_opts requires a nongeneric module struct");
    }
    let [variant] = facts.members.variants(item) else {
        return Err("transmute_opts requires exactly one actual struct variant");
    };
    let variant = facts
        .members
        .variants
        .get(variant.0 as usize)
        .ok_or("transmute_opts lost its actual variant")?;
    if variant.owner != item || variant.origin.source() != Some(declaration.origin) {
        return Err("transmute_opts fields belong to another declaration");
    }
    let fields = facts
        .members
        .fields
        .get(variant.start as usize..variant.end as usize)
        .ok_or("transmute_opts field list is incomplete")?;
    if fields.len() != 4 {
        return Err("transmute_opts requires four actual named boolean fields");
    }
    let mut slots = [usize::MAX; 4];
    for (index, field) in fields.iter().enumerate() {
        let Name::Named(name) = field.name else {
            return Err("transmute_opts requires named fields");
        };
        let position = FIELDS
            .iter()
            .position(|&expected| facts.members.names.get(name) == expected)
            .ok_or("transmute_opts has an unknown field")?;
        if slots[position] != usize::MAX
            || field.ty != TypeId::primitive(Primitive::Bool)
            || field.origin.source().is_none()
        {
            return Err("transmute_opts requires unique source-owned boolean fields");
        }
        slots[position] = index;
    }
    Ok(slots)
}

/// Result order is alignment, lifetimes, safety, validity, independent of source order.
pub(in crate::sema) fn decode(
    facts: &Declarations,
    value: ConstId,
) -> Result<[bool; 4], &'static str> {
    crate::sema::header_attributes::charge_source_contract_query(facts, 64)
        .map_err(|_| "transmutability assumption query exhausted its source budget")?;
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("transmutability assumptions require the actual source catalog")?;
    let declaration = catalog
        .validated_language(LangItem::TransmuteOpts)?
        .ok_or("transmutability assumptions require actual transmute_opts")?;
    let indices = slots(facts, declaration)?;
    let Some(ConstKind::Aggregate(id)) = facts.types.const_kind(value) else {
        return Err("transmutability assumptions require a completed aggregate const value");
    };
    let Some(Aggregate::Fields {
        ty,
        variant: None,
        values,
    }) = facts.types.aggregate_constant(id)
    else {
        return Err("transmutability assumptions require an actual struct const value");
    };
    if !matches!(facts.types.kind(ty), Some(Kind::Adt(item, GenericArgs::EMPTY))
        if item.0 == declaration.definition.0)
        || values.len() != 4
    {
        return Err("transmutability assumptions have another actual owner or arity");
    }
    let mut result = [false; 4];
    for (position, index) in indices.into_iter().enumerate() {
        let scalar = facts
            .types
            .constant(values[index])
            .ok_or("transmutability assumptions require actual boolean leaves")?;
        if scalar.ty != TypeId::primitive(Primitive::Bool) || scalar.bits > 1 {
            return Err("transmutability assumptions contain an invalid boolean leaf");
        }
        result[position] = scalar.bits == 1;
    }
    Ok(result)
}
