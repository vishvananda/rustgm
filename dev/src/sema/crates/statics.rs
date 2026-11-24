//! Imported originals map a static to its one provider allocation and completed value.
use super::{Declarations, Error, ImportedStorage, Report, error, static_symbol};
use crate::{
    literal::value::FloatBits,
    resolve::model::DefId,
    sema::{
        foreign::Global,
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
        TypeId,
        interior_mutability::{Answer, Query},
    },
};

pub(super) fn seed(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    let mut relocation_work = crate::support::crates::MAX_COMPONENTS;
    for artifact in imported.dependencies().artifacts() {
        let Some(descriptor) = artifact.source_descriptor() else {
            continue;
        };
        let rows = descriptor.rows();
        let Some(statics) = rows.statics() else {
            continue;
        };
        for header in &statics.headers {
            let definition = imported
                .generic_definition(descriptor.identity(), header.declaration)
                .map_err(error)?;
            let ty = facts.definitions[definition.0 as usize]
                .ok_or_else(|| error("imported source static has no installed type"))?;
            let (value, storage) = match header.initializer {
                s::StaticInitializer::Scalars(values) => (
                    decode_scalar_aggregate(
                        &mut Evaluator::new(report, facts),
                        rows,
                        header.ty,
                        values.slice(&statics.values).map_err(error)?,
                    )?,
                    ImportedStorage::Ordinary,
                ),
                s::StaticInitializer::Promotion(anchor) => {
                    let target = imported
                        .generic_definition(descriptor.identity(), anchor)
                        .map_err(error)?;
                    let value = super::relocated_initializers::import(
                        report,
                        facts,
                        ty,
                        target,
                        anchor,
                        &mut relocation_work,
                    )?;
                    (value, import_storage_kind(facts, ty, &mut relocation_work)?)
                }
            };
            declare(
                facts,
                descriptor.identity(),
                definition,
                header.declaration,
                ty,
                storage,
            )?;
            Evaluator::new(report, facts).complete_definition(definition, value);
        }
    }
    Ok(())
}

fn import_storage_kind(
    facts: &Declarations,
    ty: TypeId,
    remaining: &mut usize,
) -> Result<ImportedStorage, Error> {
    if !facts.aggregates.has_shared_mutation() {
        return Ok(ImportedStorage::Ordinary);
    }
    let mut query = Query::new(&facts.types, &facts.aggregates);
    let result = query.contains(ty).map_err(error)?;
    *remaining = remaining
        .checked_sub(query.work().steps)
        .ok_or_else(|| error("source static relocation exceeds its complete storage budget"))?;
    match result {
        Answer::Ordinary => Ok(ImportedStorage::Ordinary),
        Answer::Interior => Ok(ImportedStorage::Interior),
        Answer::Unresolved => Err(error(
            "source static relocation requires completed mutation policy",
        )),
    }
}

fn declare(
    facts: &mut Declarations,
    identity: &CrateIdentity,
    definition: DefId,
    declaration: crate::support::crates::owner_catalog::DeclarationRef,
    ty: TypeId,
    storage: ImportedStorage,
) -> Result<(), Error> {
    let linkage = facts.crates.get_or_insert_default();
    linkage.imported_storage.insert(definition, storage);
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
    Array(TypeRef, usize),
    Seal(usize),
}

pub(super) fn decode_scalar_aggregate(
    evaluator: &mut Evaluator<'_>,
    rows: s::ValidatedView<'_>,
    ty: TypeRef,
    mut data: &[u8],
) -> Result<Constant, Error> {
    let mut pending = vec![Visit::Value(ty)];
    let mut values = Vec::new();
    while let Some(visit) = pending.pop() {
        match visit {
            Visit::Value(ty) => {
                match s::StaticShape::new(&rows.types, &rows.roots, rows.array_lengths(), ty)
                    .map_err(error)?
                {
                    s::StaticShape::Scalar(ty) => values.push(scalar(evaluator, ty, &mut data)?),
                    s::StaticShape::WideFloat(width) => values.push(wide(width, &mut data)?),
                    s::StaticShape::Tuple(fields) => {
                        pending.push(Visit::Seal(values.len()));
                        pending.push(Visit::Tuple(fields.iter()));
                    }
                    s::StaticShape::Array(element, length) => {
                        pending.push(Visit::Seal(values.len()));
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
            Visit::Array(element, length) => {
                if length != 0 {
                    pending.push(Visit::Array(element, length - 1));
                    pending.push(Visit::Value(element));
                }
            }
            Visit::Seal(start) => {
                let value = evaluator
                    .fields(&values[start..], None)
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
