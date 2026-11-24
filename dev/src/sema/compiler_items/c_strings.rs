//! C string storage consumes its completed source type and ordinary byte tail.
use crate::{
    resolve::body_origin::DeclarationOrigin,
    sema::{
        members::{FieldId, Origin, Selection},
        model::Declarations,
    },
    support::bindings::LangItem,
    types::{ItemId, Kind, Primitive, TypeId, aggregate},
};

#[derive(Clone, Copy)]
pub(crate) struct LiteralProfile {
    pub referent: TypeId,
    pub element: TypeId,
    pub field: Selection,
}

/// The source literal rule already chose its actual immutable reference type.
/// Storage classification supplies no source body, loan or mutation permission.
pub(crate) fn literal_profile(
    facts: &Declarations,
    reference: TypeId,
) -> Result<Option<LiteralProfile>, &'static str> {
    let Some(Kind::Reference {
        mutable: false,
        referent,
        ..
    }) = facts.types.kind(reference)
    else {
        return Err("text literal storage requires its immutable reference type");
    };
    storage_profile(facts, referent)
}

pub(crate) fn storage_profile(
    facts: &Declarations,
    referent: TypeId,
) -> Result<Option<LiteralProfile>, &'static str> {
    let Some(Kind::Adt(owner, _)) = facts.types.kind(referent) else {
        return Ok(None);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(declaration) = catalog.role(LangItem::CStr)? else {
        return Ok(None);
    };
    if owner != ItemId(declaration.definition.0) {
        return Ok(None);
    }
    catalog.validated_role(LangItem::CStr)?;
    checked_storage(facts, declaration, referent).map(Some)
}

pub(super) fn imported(
    facts: &Declarations,
    declaration: super::Role,
    steps: &mut usize,
) -> Result<(), &'static str> {
    *steps = steps.saturating_add(12);
    let referent = facts
        .definitions
        .get(declaration.definition.0 as usize)
        .copied()
        .flatten()
        .ok_or("imported CStr is missing its completed nominal type")?;
    if !matches!(facts.types.kind(referent), Some(Kind::Adt(item, args)) if item.0 == declaration.definition.0
        && facts.types.arguments(args).is_some_and(|args| args.len() == 0))
    {
        return Err("imported CStr requires its actual parameterless nominal");
    }
    checked_storage(facts, declaration, referent)?;
    Ok(())
}

fn checked_storage(
    facts: &Declarations,
    declaration: super::Role,
    referent: TypeId,
) -> Result<LiteralProfile, &'static str> {
    let owner = ItemId(declaration.definition.0);
    if !facts
        .generics
        .nominal_contracts
        .contains(&declaration.definition)
    {
        return Err("C string storage requires its completed actual source declaration");
    }
    let definition = facts
        .aggregates
        .get(owner)
        .ok_or("C string storage declaration is pending")?;
    if definition.item != owner
        || definition.kind != aggregate::Kind::Struct
        || definition.repr.representation != aggregate::Representation::Transparent
        || definition.repr.packed.is_some()
        || definition.repr.align.is_some()
        || facts.aggregates.shared_mutation(owner) != Some(aggregate::SharedMutation::Ordinary)
    {
        return Err("C string storage requires its actual transparent immutable byte-tail profile");
    }
    let [variant] = facts.members.variants(owner) else {
        return Err("C string storage requires its actual single struct variant");
    };
    let record = facts
        .members
        .variants
        .get(variant.0 as usize)
        .ok_or("C string variant is missing")?;
    let origin = match declaration.origin {
        DeclarationOrigin::Source(node) => Origin::Source(node),
        DeclarationOrigin::Imported(_) => Origin::External(declaration.definition),
    };
    if record.owner != owner
        || record.ordinal != 0
        || record.origin != origin
        || facts
            .members
            .fields(*variant)
            .is_none_or(|fields| fields.len() != 1)
    {
        return Err("C string storage requires its actual single field");
    }
    let shape = facts.aggregates.shape(&facts.types, referent)?;
    let mut variants = shape.variants();
    let physical = variants.next().ok_or("C string field storage is pending")?;
    let [tail] = facts
        .types
        .list(physical.fields)
        .ok_or("C string field storage is missing")?
    else {
        return Err("C string storage requires its actual single byte-slice tail");
    };
    if variants.next().is_some() {
        return Err("C string storage has multiple actual variants");
    }
    let Some(Kind::Slice(element)) = facts.types.kind(*tail) else {
        return Err("C string storage requires its actual byte-slice tail");
    };
    if !matches!(
        facts.types.kind(element),
        Some(Kind::Primitive(Primitive::I8 | Primitive::U8))
    ) {
        return Err("C string storage requires a one-byte character element");
    }
    Ok(LiteralProfile {
        referent,
        element,
        field: Selection::Nominal(FieldId(record.start)),
    })
}
