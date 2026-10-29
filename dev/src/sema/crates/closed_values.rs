//! Shared closed primitive conversion preserves exact typed bits across metadata lanes.
use crate::{
    literal::value::FloatBits,
    sema::model::Scalar as Value,
    support::crates::{Scalar, ScalarValue},
};
pub(crate) fn decode(value: ScalarValue) -> Result<Value, &'static str> {
    let bits = value.bits();
    Ok(match value.ty() {
        Scalar::Unit => return Err("unit constant requires its typed empty value"),
        Scalar::Bool => Value::Boolean(bits != 0),
        Scalar::Char => Value::Character(
            u32::try_from(bits)
                .ok()
                .and_then(char::from_u32)
                .ok_or("invalid imported constant character")?,
        ),
        Scalar::F32 => Value::Float(FloatBits::F32(
            u32::try_from(bits).map_err(|_| "invalid imported f32 bits")?,
        )),
        Scalar::F64 => Value::Float(FloatBits::F64(
            u64::try_from(bits).map_err(|_| "invalid imported f64 bits")?,
        )),
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
        | Scalar::Isize => Value::Integer(bits),
    })
}

pub(crate) fn encode(ty: Scalar, value: Value) -> Result<ScalarValue, &'static str> {
    let bits = match (ty, value) {
        (Scalar::Bool, Value::Boolean(value)) => u128::from(value),
        (Scalar::Char, Value::Character(value)) => u128::from(u32::from(value)),
        (Scalar::F32, Value::Float(FloatBits::F32(bits))) => u128::from(bits),
        (Scalar::F64, Value::Float(FloatBits::F64(bits))) => u128::from(bits),
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
            Value::Integer(bits),
        ) => bits,
        _ => {
            return Err("constant value disagrees with its scalar export type");
        }
    };
    ScalarValue::new(ty, bits)
}
