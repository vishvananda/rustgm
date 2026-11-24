//! Imported originals map a static to its one provider allocation and completed value.
use super::relocated_initializers::{self, Relocation};
use super::{Declarations, Error, ImportedStorage, Report, error, static_symbol};
use crate::{
    literal::value::FloatBits,
    resolve::{crates::Imported, model::DefId},
    sema::{
        foreign::Global,
        members::VariantId,
        model::Scalar as PrimitiveValue,
        values::{Constant, Evaluator},
    },
    support::{
        crates::{
            CrateIdentity, Scalar, ScalarValue,
            generics::{TypeRef, source as s},
        },
        linkage::{Linkage, Naming},
    },
    syntax::item::Safety,
    types::{
        ItemId, TypeId,
        interior_mutability::{Answer, Query},
    },
};

pub(super) struct Pending {
    relocations: Vec<Relocation>,
    storage: Vec<(DefId, TypeId)>,
    remaining: usize,
}

pub(in crate::sema) fn complete_relocations(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let Some(pending) = facts
        .crates
        .as_deref_mut()
        .and_then(|linkage| linkage.pending_relocations.take())
    else {
        return Ok(());
    };
    let Pending {
        relocations,
        storage,
        mut remaining,
    } = *pending;
    for relocation in relocations {
        let value = relocated_initializers::finish(report, facts, &relocation, &mut remaining)?;
        let kind = import_storage_kind(facts, relocation.ty, &mut remaining)?;
        facts
            .crates
            .get_or_insert_default()
            .imported_storage
            .insert(relocation.definition, kind);
        Evaluator::new(report, facts).complete_definition(relocation.definition, value);
    }
    for (definition, ty) in storage {
        let mut work = 0;
        crate::sema::stored_fields::prepare_premise_free(facts, ty, &mut work)
            .map_err(Error::unlocated)?;
        remaining = remaining
            .checked_sub(work)
            .ok_or_else(|| error("source static relocation exceeds its complete storage budget"))?;
        let kind = import_storage_kind(facts, ty, &mut remaining)?;
        facts
            .crates
            .get_or_insert_default()
            .imported_storage
            .insert(definition, kind);
    }
    Ok(())
}

pub(super) fn seed(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    let mut pending = Pending {
        relocations: Vec::new(),
        storage: Vec::new(),
        remaining: crate::support::crates::MAX_COMPONENTS,
    };
    for artifact in imported.dependencies().artifacts() {
        let Some(descriptor) = artifact.source_descriptor() else {
            continue;
        };
        let rows = descriptor.rows();
        let Some(statics) = rows.statics() else {
            continue;
        };
        let source = ImportSource {
            imported,
            identity: descriptor.identity(),
            rows,
            statics,
        };
        for header in &statics.headers {
            seed_header(report, facts, source, *header, &mut pending)?;
        }
    }
    if !pending.relocations.is_empty() || !pending.storage.is_empty() {
        facts.crates.get_or_insert_default().pending_relocations = Some(Box::new(pending));
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ImportSource<'a> {
    imported: &'a Imported,
    identity: &'a CrateIdentity,
    rows: s::ValidatedView<'a>,
    statics: &'a s::StaticColumns,
}

fn seed_header(
    report: &Report,
    facts: &mut Declarations,
    source: ImportSource<'_>,
    header: s::StaticHeader,
    pending: &mut Pending,
) -> Result<(), Error> {
    let definition = source
        .imported
        .generic_definition(source.identity, header.declaration)
        .map_err(error)?;
    let ty = facts.definitions[definition.0 as usize]
        .ok_or_else(|| error("imported source static has no installed type"))?;
    let (value, storage) = match header.initializer {
        s::StaticInitializer::Scalars(values) => {
            let value = decode_scalar_aggregate(
                &mut Evaluator::new(report, facts),
                Some((source.imported, source.identity)),
                source.rows,
                header.ty,
                values.slice(&source.statics.values).map_err(error)?,
            )?;
            let storage =
                if let Some(kind) = try_import_storage_kind(facts, ty, &mut pending.remaining)? {
                    kind
                } else {
                    pending.storage.push((definition, ty));
                    ImportedStorage::Pending
                };
            (Some(value), Some(storage))
        }
        s::StaticInitializer::Promotion(anchor) => {
            let target = source
                .imported
                .generic_definition(source.identity, anchor)
                .map_err(error)?;
            let relocation = Relocation {
                definition,
                ty,
                target,
                anchor,
            };
            if let Some(value) =
                relocated_initializers::import(report, facts, &relocation, &mut pending.remaining)?
            {
                (
                    Some(value),
                    Some(import_storage_kind(facts, ty, &mut pending.remaining)?),
                )
            } else {
                pending.relocations.push(relocation);
                (None, Some(ImportedStorage::Pending))
            }
        }
        s::StaticInitializer::ProviderOwned => (None, None),
    };
    declare(
        facts,
        source.identity,
        definition,
        header.declaration,
        ty,
        storage,
    )?;
    if let Some(value) = value {
        Evaluator::new(report, facts).complete_definition(definition, value);
    }
    Ok(())
}

fn try_import_storage_kind(
    facts: &Declarations,
    ty: TypeId,
    remaining: &mut usize,
) -> Result<Option<ImportedStorage>, Error> {
    if !facts.aggregates.has_shared_mutation() {
        return Ok(Some(ImportedStorage::Ordinary));
    }
    let mut query = Query::new(&facts.types, &facts.aggregates);
    let result = query.contains(ty).map_err(error)?;
    *remaining = remaining
        .checked_sub(query.work().steps)
        .ok_or_else(|| error("source static relocation exceeds its complete storage budget"))?;
    Ok(match result {
        Answer::Ordinary => Some(ImportedStorage::Ordinary),
        Answer::Interior => Some(ImportedStorage::Interior),
        Answer::Unresolved => None,
    })
}

fn import_storage_kind(
    facts: &Declarations,
    ty: TypeId,
    remaining: &mut usize,
) -> Result<ImportedStorage, Error> {
    try_import_storage_kind(facts, ty, remaining)?
        .ok_or_else(|| error("source static relocation requires completed mutation policy"))
}

fn declare(
    facts: &mut Declarations,
    identity: &CrateIdentity,
    definition: DefId,
    declaration: crate::support::crates::owner_catalog::DeclarationRef,
    ty: TypeId,
    storage: Option<ImportedStorage>,
) -> Result<(), Error> {
    let linkage = facts.crates.get_or_insert_default();
    if let Some(storage) = storage {
        linkage.imported_storage.insert(definition, storage);
    }
    linkage
        .symbols
        .insert(definition, static_symbol(identity, declaration));
    facts
        .foreign
        .get_or_insert_default()
        .insert_global(
            definition,
            Global {
                ty,
                mutable: false,
                safety: Safety::Safe,
                naming: Naming::Rust,
                linkage: Linkage::Import,
                weak: None,
            },
        )
        .map_err(error)
}

enum Visit<'a> {
    Value(TypeRef),
    Tuple(std::slice::Iter<'a, TypeRef>),
    Struct(std::slice::Iter<'a, s::NominalField>),
    Array(TypeRef, usize),
    Seal(usize, Option<VariantId>),
}

pub(super) fn decode_scalar_aggregate(
    evaluator: &mut Evaluator<'_>,
    owner: Option<(&Imported, &CrateIdentity)>,
    rows: s::ValidatedView<'_>,
    ty: TypeRef,
    mut data: &[u8],
) -> Result<Constant, Error> {
    let table = s::StaticTypeTable {
        declarations: &rows.declarations,
        nominals: &rows.nominals,
        fields: &rows.fields,
        types: &rows.types,
        roots: &rows.roots,
        lengths: rows.array_lengths(),
    };
    let mut pending = vec![Visit::Value(ty)];
    let mut values = Vec::new();
    while let Some(visit) = pending.pop() {
        match visit {
            Visit::Value(ty) => {
                let nominal = |declaration| match rows.header(declaration)? {
                    Some(s::HeaderRef::Nominal(index)) => Ok(Some(index)),
                    _ => Ok(None),
                };
                match s::StaticShape::new(table, nominal, ty).map_err(error)? {
                    s::StaticShape::Scalar(ty) => values.push(scalar(evaluator, ty, &mut data)?),
                    s::StaticShape::WideFloat(width) => values.push(wide(width, &mut data)?),
                    s::StaticShape::Tuple(fields) => {
                        pending.push(Visit::Seal(values.len(), None));
                        pending.push(Visit::Tuple(fields.iter()));
                    }
                    s::StaticShape::Struct {
                        declaration,
                        fields,
                    } => {
                        let Some((imported, identity)) = owner else {
                            return Err(error(
                                "imported scalar constant cannot contain nominal structs",
                            ));
                        };
                        let definition = imported
                            .generic_definition(identity, declaration)
                            .map_err(error)?;
                        let [variant] = evaluator.facts().members.variants(ItemId(definition.0))
                        else {
                            return Err(error("imported source static struct lost its variant"));
                        };
                        pending.push(Visit::Seal(values.len(), Some(*variant)));
                        pending.push(Visit::Struct(fields.iter()));
                    }
                    s::StaticShape::Array(element, length) => {
                        pending.push(Visit::Seal(values.len(), None));
                        pending.push(Visit::Array(element, length));
                    }
                }
            }
            Visit::Tuple(mut fields) => {
                if let Some(&field) = fields.next() {
                    pending.push(Visit::Tuple(fields));
                    pending.push(Visit::Value(field));
                }
            }
            Visit::Struct(mut fields) => {
                if let Some(field) = fields.next() {
                    pending.push(Visit::Struct(fields));
                    pending.push(Visit::Value(field.ty));
                }
            }
            Visit::Array(element, length) => {
                if length != 0 {
                    pending.push(Visit::Array(element, length - 1));
                    pending.push(Visit::Value(element));
                }
            }
            Visit::Seal(start, variant) => {
                let value = evaluator
                    .fields(&values[start..], variant)
                    .map_err(|failure| error(failure.message()))?;
                values.truncate(start);
                values.push(value);
            }
        }
    }
    match values.as_slice() {
        [value] if data.is_empty() => Ok(*value),
        _ => Err(error(
            "imported source static initializer changed its exact shape",
        )),
    }
}

fn scalar(evaluator: &mut Evaluator<'_>, ty: Scalar, data: &mut &[u8]) -> Result<Constant, Error> {
    let bits = s::read_static_bits(data, ty.bit_width().div_ceil(8) as usize).map_err(error)?;
    let value = ScalarValue::new(ty, bits).map_err(error)?;
    if ty == Scalar::Unit {
        return evaluator
            .fields(&[], None)
            .map_err(|failure| error(failure.message()));
    }
    super::closed_values::decode(value)
        .map(Constant::scalar)
        .map_err(error)
}

fn wide(width: usize, data: &mut &[u8]) -> Result<Constant, Error> {
    let bits = s::read_static_bits(data, width).map_err(error)?;
    let float = match width {
        2 => FloatBits::F16(u16::try_from(bits).map_err(|_| error("invalid imported f16 bits"))?),
        16 => FloatBits::from_quad(bits),
        _ => return Err(error("invalid imported wide-float width")),
    };
    Ok(Constant::scalar(PrimitiveValue::Float(float)))
}
