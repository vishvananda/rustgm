//! Public named constants expose completed scalar bits without demanding runtime storage.
use super::{Declarations, DefId, Error, error, scalar};
use crate::{
    literal::value::FloatBits,
    sema::{
        model::Scalar as Value,
        values::{Reader, View},
    },
    support::crates::{Constant, ExternalDef, Scalar, ScalarValue},
};

pub(super) fn export(
    facts: &Declarations,
    definition: DefId,
    external: ExternalDef,
) -> Result<Constant, Error> {
    if facts.generics.has_parameters(definition) || facts.associated.owner(definition).is_some() {
        return Err(error(
            "generic associated values are outside the scalar metadata profile",
        ));
    }
    let ty = facts.definitions[definition.0 as usize]
        .ok_or_else(|| error("missing constant export type"))?;
    let ty = scalar(facts, ty)?;
    let reader = Reader::new(facts);
    let value = reader
        .definition(definition)
        .ok_or_else(|| error("scalar constant export requires a completed value"))?;
    Constant::new(
        external,
        value_bits(ty, reader.view(value).map_err(error)?)?,
    )
    .map_err(error)
}

pub(super) fn value_bits(ty: Scalar, view: View) -> Result<ScalarValue, Error> {
    let bits = match (ty, view) {
        (
            Scalar::Unit,
            View::Product {
                length: 0,
                variant: None,
            },
        ) => 0,
        (Scalar::Bool, View::Scalar(Value::Boolean(value))) => u128::from(value),
        (Scalar::Char, View::Scalar(Value::Character(value))) => u128::from(u32::from(value)),
        (Scalar::F32, View::Scalar(Value::Float(FloatBits::F32(bits)))) => u128::from(bits),
        (Scalar::F64, View::Scalar(Value::Float(FloatBits::F64(bits)))) => u128::from(bits),
        (
            Scalar::U8
            | Scalar::U16
            | Scalar::U32
            | Scalar::U64
            | Scalar::U128
            | Scalar::Usize
            | Scalar::I8
            | Scalar::I16
            | Scalar::I32
            | Scalar::I64
            | Scalar::I128
            | Scalar::Isize,
            View::Scalar(Value::Integer(bits)),
        ) => bits,
        _ => {
            return Err(error(
                "constant value disagrees with its scalar export type",
            ));
        }
    };
    ScalarValue::new(ty, bits).map_err(error)
}
