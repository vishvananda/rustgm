//! Remapped source nominal rows prepare both ordinary member and storage owners.
use super::{Declarations, Error, Headers, error, mapped_source};
use crate::{
    resolve::model::{Access, DefId},
    sema::members::{ExternalVariant, Field, Name, Origin, PreparedMembers},
    support::{
        crates::{RecordRepr, generics::source as s},
        retained,
    },
    syntax::item::Shape,
    types::{
        ItemId,
        aggregate::{self, ImportedEnum, NullableEnum, PreparedShapes},
    },
};
pub(super) struct Prepared {
    pub members: PreparedMembers,
    pub shapes: PreparedShapes,
    pub owners: Vec<DefId>,
}
pub(super) fn prepare(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
    budget: &mut Headers,
) -> Result<Option<Prepared>, Error> {
    let rows = context.descriptor.rows();
    if rows.nominals.is_empty() {
        return Ok(None);
    }
    budget.charge(
        rows.nominals
            .len()
            .saturating_add(rows.fields.len())
            .saturating_add(rows.enums().variants.len())
            .saturating_add(rows.enums().fields.len()),
    )?;
    budget.reserve(
        rows.nominals
            .len()
            .saturating_mul(4 * (size_of::<DefId>() + size_of::<(ItemId, Vec<Field>)>())),
    )?;
    let mut members = Vec::with_capacity(rows.nominals.len());
    let mut shapes = Vec::with_capacity(rows.nominals.len());
    let mut owners = Vec::with_capacity(rows.nominals.len());
    let (mut enum_members, mut enum_shapes) = (Vec::new(), Vec::new());
    let constructors = super::definition_cells::struct_constructors(rows, budget)?;
    for (row, constructor) in rows.nominals.iter().zip(constructors) {
        let owner = context.definition(row.declaration, budget)?;
        if let Ok(index) = rows
            .enums()
            .headers
            .binary_search_by_key(&row.declaration.0, |header| header.declaration.0)
        {
            let (members, shape) = enum_rows(
                context,
                graph,
                facts,
                budget,
                owner,
                &rows.enums().headers[index],
            )?;
            enum_members.push(members);
            enum_shapes.push(shape);
            owners.push(owner);
            continue;
        }
        let raw_fields = row.fields.slice(&rows.fields).map_err(error)?;
        let (fields, types) = if constructor.is_some() {
            positional_fields(graph, budget, owner, raw_fields)?
        } else {
            record_fields(graph, facts, budget, owner, raw_fields)?
        };
        let fields_type = facts.types.intern_list(&types).map_err(error)?;
        let repr = aggregate::Repr {
            representation: match row.repr {
                RecordRepr::Rust => aggregate::Representation::Rust,
                RecordRepr::C => aggregate::Representation::C,
            },
            ..aggregate::Repr::default()
        };
        // Version thirteen: a tuple or unit struct publishes its single field
        // group with its constructor, as an enum publishes each variant's.
        if let Some(constructor) = constructor {
            let shape = if fields.is_empty() {
                Shape::Unit
            } else {
                Shape::Tuple
            };
            let constructor = context.definition(constructor, budget)?;
            enum_members.push((
                ItemId(owner.0),
                vec![ExternalVariant {
                    shape,
                    fields,
                    constructor,
                }],
            ));
        } else {
            members.push((ItemId(owner.0), fields));
        }
        shapes.push((ItemId(owner.0), repr, fields_type));
        owners.push(owner);
    }
    let mut pay = |steps, bytes| {
        budget
            .charge(steps)
            .map_err(|_| "source nominal preparation exceeded header work")?;
        budget
            .reserve(bytes)
            .map_err(|_| "source nominal preparation exceeded header storage")
    };
    let members = facts
        .members
        .prepare_external_nominals(members, enum_members, &mut pay)
        .map_err(error)?;
    let shapes = facts
        .aggregates
        .prepare_imported_nominals(&facts.types, &shapes, &enum_shapes, pay)
        .map_err(error)?;
    Ok(Some(Prepared {
        members,
        shapes,
        owners,
    }))
}
/// A record's named fields with their canonical types and access.
fn record_fields(
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
    budget: &mut Headers,
    owner: DefId,
    raw_fields: &[s::NominalField],
) -> Result<(Vec<Field>, Vec<crate::types::TypeId>), Error> {
    let mut fields = Vec::with_capacity(raw_fields.len());
    let mut types = Vec::with_capacity(raw_fields.len());
    budget.reserve(
        raw_fields
            .len()
            .saturating_mul(4 * (size_of::<Field>() + size_of::<crate::types::TypeId>())),
    )?;
    for field in raw_fields {
        budget.charge(field.name.len().saturating_add(1))?;
        budget.reserve(field.name.len().saturating_mul(4))?;
        let ty = graph.ty(field.ty)?;
        let name = Name::Named(
            facts
                .members
                .names
                .intern_exact_paid(&field.name, &mut |growth| {
                    budget
                        .charge(growth.work)
                        .map_err(|_| "source nominal name work exceeded header budget")?;
                    budget
                        .reserve(growth.bytes)
                        .map_err(|_| "source nominal name storage exceeded header budget")
                })
                .map_err(error)?,
        );
        fields.push(Field {
            origin: Origin::External(owner),
            name,
            ty,
            access: if field.visibility
                == crate::support::crates::generics::source::ModuleVisibility::Public
            {
                Access::Public
            } else {
                Access::ExternalPrivate
            },
        });
        types.push(ty);
    }
    Ok((fields, types))
}
/// A tuple struct's fields, named by their ordinals, with their canonical
/// types and access.
fn positional_fields(
    graph: &mapped_source::Graph,
    budget: &mut Headers,
    owner: DefId,
    raw_fields: &[s::NominalField],
) -> Result<(Vec<Field>, Vec<crate::types::TypeId>), Error> {
    budget.charge(raw_fields.len())?;
    budget.reserve(
        raw_fields
            .len()
            .saturating_mul(4 * (size_of::<Field>() + size_of::<crate::types::TypeId>())),
    )?;
    let mut fields = Vec::with_capacity(raw_fields.len());
    let mut types = Vec::with_capacity(raw_fields.len());
    for (index, field) in (0_u32..).zip(raw_fields) {
        let ty = graph.ty(field.ty)?;
        fields.push(Field {
            origin: Origin::External(owner),
            name: Name::Index(index),
            ty,
            access: if field.visibility == s::ModuleVisibility::Public {
                Access::Public
            } else {
                Access::ExternalPrivate
            },
        });
        types.push(ty);
    }
    Ok((fields, types))
}
/// One validated enum header becomes its ordered variants, constructor names,
/// completed discriminants and the re-derived nullable layout policy.
fn enum_rows(
    context: &mapped_source::Context<'_>,
    graph: &mapped_source::Graph,
    facts: &mut Declarations,
    budget: &mut Headers,
    owner: DefId,
    header: &s::EnumHeader,
) -> Result<((ItemId, Vec<ExternalVariant>), ImportedEnum), Error> {
    let rows = context.descriptor.rows();
    let source = header
        .variants
        .slice(&rows.enums().variants)
        .map_err(error)?;
    budget.reserve(
        source
            .len()
            .saturating_mul(4 * (size_of::<ExternalVariant>() + size_of::<aggregate::Variant>())),
    )?;
    let mut members = Vec::with_capacity(source.len());
    let mut variants = Vec::with_capacity(source.len());
    for variant in source {
        let payload = variant.fields.slice(&rows.enums().fields).map_err(error)?;
        budget.charge(payload.len().saturating_add(1))?;
        budget.reserve(
            payload
                .len()
                .saturating_mul(4 * (size_of::<Field>() + size_of::<crate::types::TypeId>())),
        )?;
        let mut fields = Vec::with_capacity(payload.len());
        let mut types = Vec::with_capacity(payload.len());
        for (index, field) in (0_u32..).zip(payload) {
            let ty = graph.ty(field.ty)?;
            fields.push(Field {
                origin: Origin::External(owner),
                name: Name::Index(index),
                ty,
                access: Access::Public,
            });
            types.push(ty);
        }
        members.push(ExternalVariant {
            shape: match variant.shape {
                s::VariantShape::Unit => Shape::Unit,
                s::VariantShape::Tuple => Shape::Tuple,
            },
            fields,
            constructor: context.definition(variant.declaration, budget)?,
        });
        variants.push(aggregate::Variant {
            fields: facts.types.intern_list(&types).map_err(error)?,
            discriminant: variant.discriminant,
        });
    }
    let tag = header
        .tag
        .map(|tag| {
            crate::sema::crates::scalar_primitive(tag)
                .ok_or_else(|| error("source enum representation has no integer primitive"))
        })
        .transpose()?;
    let nullable = nullable(source, header.tag.is_some());
    Ok((
        (ItemId(owner.0), members),
        ImportedEnum {
            item: ItemId(owner.0),
            repr: aggregate::Repr {
                tag,
                ..aggregate::Repr::default()
            },
            variants,
            nullable,
        },
    ))
}
/// The provider's source nullable rule, re-derived from the validated header:
/// default representation and one empty plus one single-field variant. Their
/// discriminants are implicit: validation rejects explicit data discriminants
/// without an integer tag.
fn nullable(variants: &[s::VariantHeader], tagged: bool) -> Option<NullableEnum> {
    let [first, second] = variants else {
        return None;
    };
    if tagged {
        return None;
    }
    let arity = |row: &s::VariantHeader| row.fields.end.saturating_sub(row.fields.start);
    let (empty, payload) = match (arity(first), arity(second)) {
        (0, 1) => (0, 1),
        (1, 0) => (1, 0),
        _ => return None,
    };
    Some(NullableEnum {
        empty,
        payload,
        field: 0,
        fallback_tag: Some(crate::types::Primitive::U8),
    })
}

pub(super) fn reserve_owners(
    facts: &mut Declarations,
    count: usize,
    budget: &mut Headers,
) -> Result<(), Error> {
    if count == 0 {
        return Ok(());
    }
    let linkage = facts
        .crates
        .as_deref_mut()
        .ok_or_else(|| error("source nominal owner destination linkage is missing"))?;
    let mut growth = retained::Growth::map::<DefId, ()>(
        linkage.imported_records.len(),
        linkage.imported_records.capacity(),
        count,
    );
    growth.include(retained::Growth::map::<DefId, ()>(
        facts.generics.nominal_contracts.len(),
        facts.generics.nominal_contracts.capacity(),
        count,
    ));
    budget.charge(growth.work)?;
    budget.reserve(growth.bytes)?;
    linkage
        .imported_records
        .try_reserve(count)
        .map_err(|_| error("source nominal linkage destination reservation failed"))?;
    facts
        .generics
        .nominal_contracts
        .try_reserve(count)
        .map_err(|_| error("source nominal contract destination reservation failed"))?;
    Ok(())
}
