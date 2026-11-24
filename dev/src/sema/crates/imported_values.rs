//! Imported closed constant values have typed facts but no source initializer or storage.
use super::{Declarations, Error, Report, error, scalar};
use crate::{
    literal::value::FloatBits,
    resolve::model::DefId,
    sema::{
        associated::OwnerKind,
        model::Scalar as PrimitiveValue,
        values::{Constant, Evaluator, Instance},
    },
    support::crates::{Scalar, ScalarValue, generics::source as s},
    types::GenericArgs,
};

#[derive(Clone, Copy)]
enum NamedValue<'rows> {
    Unit,
    Scalar(PrimitiveValue),
    InlineTuple {
        rows: s::ValidatedView<'rows>,
        value: s::ConstValue,
        width: usize,
    },
}

struct SourceValues<'rows> {
    constants: Vec<(DefId, NamedValue<'rows>)>,
    promotions: Vec<super::promotions::Imported<'rows>>,
}

impl NamedValue<'_> {
    fn decode(value: ScalarValue) -> Result<Self, Error> {
        if value.ty() == Scalar::Unit {
            Ok(Self::Unit)
        } else {
            convert(value).map(Self::Scalar).map_err(error)
        }
    }
}

pub(super) fn seed(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let Some(imported) = &report.resolution.crates else {
        return Ok(());
    };
    let SourceValues {
        constants: source,
        promotions,
    } = source_constants(imported)?;
    if imported.constants().next().is_none() && source.is_empty() && promotions.is_empty() {
        return Ok(());
    }
    for (definition, value) in imported.constants() {
        facts.definitions[definition.0 as usize] = Some(scalar(value.value().ty()));
        facts
            .crates
            .get_or_insert_default()
            .imported_constants
            .insert(definition);
    }
    // A source constant's cell is already its installed header type.
    for definition in source
        .iter()
        .map(|(definition, _)| *definition)
        .chain(promotions.iter().map(|promotion| promotion.definition))
    {
        if facts.associated.owner(definition).is_none() {
            facts
                .crates
                .get_or_insert_default()
                .imported_constants
                .insert(definition);
        }
    }
    let mut evaluator = Evaluator::new(report, facts);
    for (definition, value) in imported.constants() {
        complete(
            &mut evaluator,
            definition,
            NamedValue::decode(value.value())?,
        )?;
    }
    for (definition, value) in source {
        complete(&mut evaluator, definition, value)?;
    }
    super::promotions::seed(report, facts, &promotions)
}
/// Each source artifact's constant rows retain canonical bits of their
/// declared types. Wide floats use source recipes instead of ABI scalar tags.
fn source_constants(
    imported: &crate::resolve::crates::Imported,
) -> Result<SourceValues<'_>, Error> {
    let mut constants = Vec::new();
    let mut promotions = Vec::new();
    for artifact in imported.dependencies().artifacts() {
        let Some(descriptor) = artifact.source_descriptor() else {
            continue;
        };
        let rows = descriptor.rows();
        for (declaration, row) in rows.constant_rows() {
            // Associated declarations and their templates have no completed
            // value to seed. Evaluation receives their selected owner later.
            if !row.body.evaluated() && descriptor.profile().declaration_policies() {
                continue;
            }
            let definition = imported
                .generic_definition(descriptor.identity(), declaration)
                .map_err(error)?;
            if let s::ConstantBody::PromotedReference { anchor, handlers } = row.body {
                promotions.push(super::promotions::Imported {
                    provider: artifact.identity(),
                    rows,
                    definition,
                    declaration,
                    anchor,
                    handlers,
                });
                continue;
            }
            let s::ConstantBody::Evaluated(s::ConstTerm::Value(value)) = row.body else {
                return Err(error("source constant seed lost its validated value"));
            };
            let value = match rows.types.get(value.ty.0 as usize) {
                Some(&s::Type::Scalar(ty)) => {
                    NamedValue::decode(ScalarValue::new(ty, value.bits).map_err(error)?)?
                }
                Some(s::Type::F16) => {
                    let bits = u16::try_from(value.bits)
                        .map_err(|_| error("invalid imported f16 bits"))?;
                    NamedValue::Scalar(PrimitiveValue::Float(FloatBits::F16(bits)))
                }
                Some(s::Type::F128) => {
                    NamedValue::Scalar(PrimitiveValue::Float(FloatBits::from_quad(value.bits)))
                }
                Some(&s::Type::Tuple(fields)) => NamedValue::InlineTuple {
                    rows,
                    width: s::inline_tuple_width(&rows.types, &rows.roots, fields)
                        .map_err(error)?,
                    value,
                },
                _ => return Err(error("source constant seed lost its validated value")),
            };
            constants.push((definition, value));
        }
    }
    Ok(SourceValues {
        constants,
        promotions,
    })
}
/// A unit value is its typed empty aggregate; every other value is its scalar.
fn complete(
    evaluator: &mut Evaluator<'_>,
    definition: DefId,
    value: NamedValue<'_>,
) -> Result<(), Error> {
    let value = match value {
        NamedValue::Unit => evaluator
            .fields(&[], None)
            .map_err(|e| error(e.message()))?,
        NamedValue::Scalar(value) => Constant::scalar(value),
        NamedValue::InlineTuple { rows, value, width } => {
            let bytes = value.bits.to_le_bytes();
            super::statics::decode_scalar_aggregate(
                evaluator,
                None,
                rows,
                value.ty,
                &bytes[..width],
            )?
        }
    };
    complete_value(evaluator, definition, value)
}

pub(super) fn complete_value(
    evaluator: &mut Evaluator<'_>,
    definition: DefId,
    value: Constant,
) -> Result<(), Error> {
    if let Some(id) = evaluator.facts().associated.owner(definition) {
        let facts = evaluator.facts();
        if !facts
            .associated
            .get(id)
            .is_some_and(|owner| matches!(owner.kind, OwnerKind::ImportedInherent(_)))
        {
            return Err(error(
                "source constant value requires its installed inherent owner",
            ));
        }
        let ty = facts
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .ok_or_else(|| error("source inherent constant lost its declared type"))?;
        evaluator
            .complete_associated(
                Instance {
                    definition,
                    arguments: GenericArgs::EMPTY,
                },
                ty,
                value,
            )
            .map_err(|failure| error(failure.message()))?;
    } else {
        evaluator.complete_definition(definition, value);
    }
    Ok(())
}
pub(super) use super::closed_values::decode as convert;
