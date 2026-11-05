//! Remapped source nominal rows prepare both ordinary member and storage owners.
use super::{Declarations, Error, Headers, error, mapped_source};
use crate::{
    resolve::model::{Access, DefId},
    sema::members::{Field, Name, Origin, PreparedMembers},
    support::{crates::RecordRepr, retained},
    types::{
        ItemId,
        aggregate::{self, PreparedShapes},
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
    budget.charge(rows.nominals.len().saturating_add(rows.fields.len()))?;
    budget.reserve(
        rows.nominals
            .len()
            .saturating_mul(4 * (size_of::<DefId>() + size_of::<(ItemId, Vec<Field>)>())),
    )?;
    let mut members = Vec::with_capacity(rows.nominals.len());
    let mut shapes = Vec::with_capacity(rows.nominals.len());
    let mut owners = Vec::with_capacity(rows.nominals.len());
    for row in &rows.nominals {
        let owner = context.definition(row.declaration, budget)?;
        let raw_fields = row.fields.slice(&rows.fields).map_err(error)?;
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
        let fields_type = facts.types.intern_list(&types).map_err(error)?;
        let repr = aggregate::Repr {
            representation: match row.repr {
                RecordRepr::Rust => aggregate::Representation::Rust,
                RecordRepr::C => aggregate::Representation::C,
            },
            ..aggregate::Repr::default()
        };
        members.push((ItemId(owner.0), fields));
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
        .prepare_external_batch(members, &mut pay)
        .map_err(error)?;
    let shapes = facts
        .aggregates
        .prepare_imported_structs(&shapes, pay)
        .map_err(error)?;
    Ok(Some(Prepared {
        members,
        shapes,
        owners,
    }))
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
