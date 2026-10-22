//! Remap a shared artifact graph once, after reserving every closed nominal identity.
use super::{Declarations, Error, error, type_transport};
use crate::support::hash::HashMap;
use crate::{
    resolve::{
        crates::Imported,
        model::{Access, DefId},
    },
    sema::members::{Field, Name, Origin},
    support::crates::{CrateIdentity, Function, Nominals, Record, RecordRepr},
    types::{GenericArgs, ItemId, Kind, TypeId, aggregate},
};

#[derive(Default)]
pub(super) struct Prepared {
    graphs: HashMap<CrateIdentity, Vec<TypeId>>,
}
impl Prepared {
    pub(super) fn types(&self, owner: &CrateIdentity) -> Result<&[TypeId], Error> {
        self.graphs
            .get(owner)
            .map(Vec::as_slice)
            .ok_or_else(|| error("missing imported shared type graph"))
    }
    pub(super) fn signature(
        &self,
        facts: &mut Declarations,
        function: &Function,
    ) -> Result<TypeId, Error> {
        if let Some(types) = self.graphs.get(function.definition().owner()) {
            type_transport::import_roots(facts, function.signature(), types)
        } else {
            type_transport::import(facts, function.signature())
        }
    }
}
pub(super) fn prepare(imported: &Imported, facts: &mut Declarations) -> Result<Prepared, Error> {
    let mut prepared = Prepared::default();
    for artifact in imported.dependencies().artifacts() {
        let Some(nominals) = artifact.nominals() else {
            continue;
        };
        let identities = reserve(imported, facts, nominals)?;
        let types = type_transport::import_graph(facts, nominals.types(), &identities)?;
        for record in nominals.records() {
            let owner = imported
                .definition(record.definition())
                .ok_or_else(|| error("missing imported record definition"))?;
            publish(facts, owner, record, &types)?;
        }
        prepared.graphs.insert(artifact.identity().clone(), types);
    }
    for (owner, _) in imported.records() {
        if crate::sema::nominals::sizes::prepare(facts, ItemId(owner.0))
            .map_err(error)?
            .is_some()
        {
            return Err(error("imported record size depends on incomplete fields"));
        }
    }
    Ok(prepared)
}
fn reserve(
    imported: &Imported,
    facts: &mut Declarations,
    nominals: &Nominals,
) -> Result<Vec<TypeId>, Error> {
    let mut types = Vec::with_capacity(nominals.records().len());
    for record in nominals.records() {
        let definition = imported
            .definition(record.definition())
            .ok_or_else(|| error("missing imported record identity"))?;
        let ty = facts
            .types
            .intern(Kind::Adt(ItemId(definition.0), GenericArgs::EMPTY))
            .map_err(error)?;
        let slot = facts
            .definitions
            .get_mut(definition.0 as usize)
            .ok_or_else(|| error("unknown imported record definition"))?;
        if slot.replace(ty).is_some() {
            return Err(error("imported record type is already published"));
        }
        types.push(ty);
    }
    Ok(types)
}
fn publish(
    facts: &mut Declarations,
    owner: DefId,
    record: &Record,
    types: &[TypeId],
) -> Result<(), Error> {
    let mut fields = Vec::with_capacity(record.fields().len());
    let mut field_types = Vec::with_capacity(record.fields().len());
    for field in record.fields() {
        let ty = *types
            .get(usize::from(field.ty().0))
            .ok_or_else(|| error("missing imported field type"))?;
        let name = Name::Named(facts.members.names.intern(field.name()).map_err(error)?);
        fields.push(Field {
            origin: Origin::External(owner),
            name,
            ty,
            access: if field.public() {
                Access::Public
            } else {
                Access::ExternalPrivate
            },
        });
        field_types.push(ty);
    }
    let item = ItemId(owner.0);
    facts
        .members
        .insert_external(item, &fields)
        .map_err(error)?;
    let fields = facts.types.intern_list(&field_types).map_err(error)?;
    facts
        .aggregates
        .insert(
            item,
            aggregate::Kind::Struct,
            aggregate::Repr {
                representation: match record.repr() {
                    RecordRepr::Rust => aggregate::Representation::Rust,
                    RecordRepr::C => aggregate::Representation::C,
                },
                ..aggregate::Repr::default()
            },
            &[aggregate::Variant {
                fields,
                discriminant: 0,
            }],
        )
        .map_err(error)?;
    facts.generics.nominal_contracts.insert(owner);
    let linkage = facts.crates.get_or_insert_default();
    linkage.imported_records.insert(owner);
    linkage.import_work.records += 1;
    linkage.import_work.fields += field_types.len();
    Ok(())
}
