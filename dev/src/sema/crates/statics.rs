//! Imported originals map a static to its one provider allocation and completed value.
use super::{Declarations, Error, Report, error, imported_values, static_symbol};
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
};

pub(super) fn seed(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
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
            declare(facts, descriptor.identity(), definition, *header)?;
            let data = header.initializer.slice(&statics.values).map_err(error)?;
            let mut evaluator = Evaluator::new(report, facts);
            let value = decode(&mut evaluator, rows, header.ty, data)?;
            evaluator.complete_definition(definition, value);
        }
    }
    Ok(())
}

fn declare(
    facts: &mut Declarations,
    identity: &CrateIdentity,
    definition: DefId,
    header: s::StaticHeader,
) -> Result<(), Error> {
    let ty = facts
        .definitions
        .get(definition.0 as usize)
        .copied()
        .flatten()
        .ok_or_else(|| error("imported source static has no installed type"))?;
    let linkage = facts.crates.get_or_insert_default();
    linkage.imported_storage.insert(definition, false);
    linkage
        .symbols
        .insert(definition, static_symbol(identity, header.declaration));
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

fn decode(
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
    imported_values::convert(value)
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
