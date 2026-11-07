//! Full representing arguments select a semantic field, never a physical-layout shortcut.
use super::{Declarations, LangItem, Readiness};
use crate::types::{
    ItemId, Kind, Primitive, TypeId, aggregate,
    arguments::{Argument, ConstKind},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::sema) enum Shape {
    Tuple(u32),
    Struct(ItemId, u32),
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::sema) struct Description {
    pub base: TypeId,
    pub field: TypeId,
    pub shape: Shape,
}

#[derive(Clone, Copy)]
pub(crate) struct Coordinates {
    pub base: TypeId,
    pub variant: u32,
    pub field: u32,
}

pub(super) enum Application {
    Absent,
    Deferred,
    Ready(Coordinates),
}

/// Coordinate data does not establish the ordinary Field implementation proof.
pub(super) fn application(facts: &Declarations, subject: TypeId) -> Result<Application, String> {
    let Some(Kind::Adt(item, arguments)) = facts.types.kind(subject) else {
        return Ok(
            if matches!(
                facts.types.kind(subject),
                Some(Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..))
            ) {
                Application::Deferred
            } else {
                Application::Absent
            },
        );
    };
    let actual = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::FieldRepresentingType))
        .ok_or("Field requires its actual representing declaration")?;
    if item.0 != actual.definition.0 {
        return Ok(Application::Absent);
    }
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("Field arguments are missing")?;
    let (
        Some(Argument::Type(base)),
        Some(Argument::Const(variant)),
        Some(Argument::Const(field)),
        None,
    ) = (
        arguments.next(),
        arguments.next(),
        arguments.next(),
        arguments.next(),
    )
    else {
        return Err("Field representing application requires type/u32/u32 arguments".into());
    };
    let Some(variant) = coordinate(&facts.types, variant)? else {
        return Ok(Application::Deferred);
    };
    let Some(field) = coordinate(&facts.types, field)? else {
        return Ok(Application::Deferred);
    };
    Ok(Application::Ready(Coordinates {
        base,
        variant,
        field,
    }))
}

pub(super) fn describe(facts: &mut Declarations, subject: TypeId) -> Result<Readiness, String> {
    crate::sema::header_attributes::charge_source_contract_query(facts, 6)
        .map_err(|e| e.to_string())?;
    let Coordinates {
        base,
        variant,
        field,
    } = match application(facts, subject)? {
        Application::Absent => return Ok(Readiness::Absent),
        Application::Deferred => return Ok(Readiness::Deferred),
        Application::Ready(coordinates) => coordinates,
    };
    if variant != 0 {
        return Ok(Readiness::Absent);
    }
    match facts
        .types
        .kind(base)
        .ok_or("Field base is outside its type store")?
    {
        Kind::Adt(item, _) => structure(facts, base, item, field),
        Kind::Tuple(list) => {
            let Some(field_ty) = facts
                .types
                .list(list)
                .and_then(|fields| fields.get(field as usize))
                .copied()
            else {
                return Ok(Readiness::Absent);
            };
            Ok(Readiness::Ready(Description {
                base,
                field: field_ty,
                shape: Shape::Tuple(field),
            }))
        }
        Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..) => Ok(Readiness::Deferred),
        _ => Ok(Readiness::Absent),
    }
}

fn structure(
    facts: &mut Declarations,
    base: TypeId,
    item: ItemId,
    field: u32,
) -> Result<Readiness, String> {
    let Some(definition) = facts.aggregates.get(item) else {
        return Ok(Readiness::Fields(item));
    };
    if definition.kind != aggregate::Kind::Struct || definition.repr.packed.is_some() {
        return Ok(Readiness::Absent);
    }
    let selected = facts
        .aggregates
        .variants(definition)
        .first()
        .and_then(|variant| facts.types.list(variant.fields));
    if selected.is_none_or(|fields| field as usize >= fields.len()) {
        return Ok(Readiness::Absent);
    }
    let mut fields = facts.aggregates.semantic_fields(&facts.types, base)?;
    let pay = crate::sema::header_attributes::source_payments(facts.must_use_contracts.as_deref())
        .map_err(|e| e.to_string())?;
    let field_ty = fields.field_paid(&mut facts.types, 0, field as usize, &|work, bytes| {
        pay.charge(work, bytes)
    })?;
    Ok(Readiness::Ready(Description {
        base,
        field: field_ty,
        shape: Shape::Struct(item, field),
    }))
}

fn coordinate(
    types: &crate::types::Store,
    id: crate::types::arguments::ConstId,
) -> Result<Option<u32>, String> {
    match types
        .const_kind(id)
        .ok_or("Field coordinate is outside its store")?
    {
        ConstKind::Parameter(_) => Ok(None),
        ConstKind::Value(value) if value.ty == TypeId::primitive(Primitive::U32) => {
            u32::try_from(value.bits)
                .map(Some)
                .map_err(|_| "Field coordinate exceeds u32".into())
        }
        _ => Err("Field coordinate requires its actual evaluated u32 type/value".into()),
    }
}
