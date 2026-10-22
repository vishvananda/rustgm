//! Imported storage uses real external declarations and completed values, never fake bodies.
use super::{Declarations, Error, Report, error, imported_records, imported_values};
use crate::{
    sema::{
        foreign,
        values::{Constant, Evaluator},
    },
    support::{
        crates::{Scalar, StorageOrigin, StoredKind, Symbol},
        linkage::{Linkage, Naming},
    },
    syntax::item::Safety,
    types::{ItemId, Kind, TypeId},
};

pub(super) fn seed(
    report: &Report,
    facts: &mut Declarations,
    graphs: &imported_records::Prepared,
) -> Result<(), Error> {
    declarations(report, facts, graphs)?;
    values(report, facts, graphs)
}
fn declarations(
    report: &Report,
    facts: &mut Declarations,
    graphs: &imported_records::Prepared,
) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    for (definition, entry) in imported.statics() {
        let ty = graphs.types(entry.definition().owner())?[usize::from(entry.ty().0)];
        facts.definitions[definition.0 as usize] = Some(ty);
        if matches!(entry.origin(), StorageOrigin::Defined(_)) {
            facts
                .crates
                .get_or_insert_default()
                .imported_storage
                .insert(definition, entry.mutable());
        }
        if entry.mutable() {
            std::sync::Arc::make_mut(facts.mutable_statics.get_or_insert_default())
                .insert(definition);
        }
        let registry = facts.foreign.get_or_insert_default();
        let naming = match entry.symbol() {
            Symbol::Rust => {
                facts
                    .crates
                    .get_or_insert_default()
                    .symbols
                    .insert(definition, entry.definition().rust_symbol());
                Naming::Rust
            }
            Symbol::Exact(name) => Naming::Exact(registry.names_mut().intern(name).map_err(error)?),
        };
        registry
            .insert_global(
                definition,
                foreign::Global {
                    ty,
                    mutable: entry.mutable(),
                    safety: if matches!(entry.origin(), StorageOrigin::Foreign { unsafe_: true }) {
                        Safety::Unsafe
                    } else {
                        Safety::Safe
                    },
                    naming,
                    linkage: Linkage::Import,
                },
            )
            .map_err(error)?;
    }
    Ok(())
}
fn values(
    report: &Report,
    facts: &mut Declarations,
    graphs: &imported_records::Prepared,
) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    for artifact in imported.dependencies().artifacts() {
        let Some(storage) = artifact.storage() else {
            continue;
        };
        let types = graphs.types(artifact.identity())?;
        let mut values = Vec::with_capacity(storage.values().len());
        let mut evaluator = Evaluator::new(report, facts);
        for value in storage.values() {
            let ty = types[usize::from(value.ty.0)];
            let value = match &value.kind {
                StoredKind::Scalar(value) if value.ty() != Scalar::Unit => {
                    Constant::scalar(imported_values::convert(*value).map_err(error)?)
                }
                StoredKind::Scalar(_) => evaluator
                    .fields(&[], None)
                    .map_err(|e| error(e.message()))?,
                StoredKind::Product(fields) => {
                    let fields = fields
                        .iter()
                        .map(|id| values[id.0 as usize])
                        .collect::<Vec<_>>();
                    let variant = variant(evaluator.facts(), ty)?;
                    evaluator
                        .fields(&fields, variant)
                        .map_err(|e| error(e.message()))?
                }
                StoredKind::Reference { target, fields } => {
                    let entry = &storage.statics()[target.0 as usize];
                    let definition = imported
                        .definition(entry.definition())
                        .ok_or_else(|| error("missing imported static identity"))?;
                    let mut ty = types[usize::from(entry.ty().0)];
                    let mut address = evaluator
                        .static_address(definition, ty)
                        .map_err(|e| error(e.message()))?;
                    for &index in fields {
                        ty = crate::sema::values::Reader::new(evaluator.facts())
                            .stored_child(ty, None, u64::from(index))
                            .map_err(error)?;
                        address = evaluator
                            .projection(address, u64::from(index), ty)
                            .map_err(|e| error(e.message()))?;
                    }
                    address.restrict_shared()
                }
            };
            values.push(value);
        }
        for entry in storage.statics() {
            if let StorageOrigin::Defined(value) = entry.origin() {
                let definition = imported
                    .definition(entry.definition())
                    .ok_or_else(|| error("missing imported static initializer identity"))?;
                evaluator.complete_definition(definition, values[value.0 as usize]);
            }
        }
    }
    Ok(())
}
fn variant(
    facts: &Declarations,
    ty: TypeId,
) -> Result<Option<crate::sema::members::VariantId>, Error> {
    if ty == TypeId::UNIT {
        return Ok(None);
    }
    let Some(Kind::Adt(ItemId(owner), _)) = facts.types.kind(ty) else {
        return Err(error("imported product lacks nominal storage"));
    };
    let [variant] = facts.members.variants(ItemId(owner)) else {
        return Err(error("imported product lacks one field group"));
    };
    Ok(Some(*variant))
}
