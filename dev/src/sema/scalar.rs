//! Built-in scalar selection and literal bits, independent of syntax traversal.
use super::model::{Cast, Scalar};
use crate::{
    lex::token::Punctuation as Op,
    literal::{
        Value,
        value::{FloatBits, FloatType, IntegerType},
    },
    syntax::ast::Unary,
    types::Primitive,
};
pub(super) fn unary(op: Unary, p: Primitive) -> bool {
    match op {
        Unary::Negate => p.is_float() || p.integer().is_some_and(|(signed, _)| signed),
        Unary::Not => p == Primitive::Bool || p.integer().is_some(),
        _ => false,
    }
}
pub(super) fn binary(op: Op, a: Primitive, b: Primitive) -> bool {
    match op {
        Op::AndAnd | Op::OrOr => a == Primitive::Bool && b == Primitive::Bool,
        Op::ShiftLeft | Op::ShiftRight => a.integer().is_some() && b.integer().is_some(),
        Op::And | Op::Or | Op::Caret => a == b && (a.integer().is_some() || a == Primitive::Bool),
        Op::EqualEqual
        | Op::BangEqual
        | Op::Less
        | Op::Greater
        | Op::LessEqual
        | Op::GreaterEqual => a == b && a != Primitive::Str,
        Op::Plus | Op::Minus | Op::Star | Op::Slash | Op::Percent => {
            a == b && (a.integer().is_some() || a.is_float())
        }
        _ => false,
    }
}
pub(super) fn cast(from: Primitive, to: Primitive) -> Option<Cast> {
    Some(if from.integer().is_some() && to.integer().is_some() {
        Cast::IntegerToInteger
    } else if from.integer().is_some() && to.is_float() {
        Cast::IntegerToFloat
    } else if from.is_float() && to.integer().is_some() {
        Cast::FloatToInteger
    } else if from.is_float() && to.is_float() {
        Cast::FloatToFloat
    } else if from == Primitive::Bool && to.integer().is_some() {
        Cast::BoolToInteger
    } else if from == Primitive::Char && to.integer().is_some() {
        Cast::CharToInteger
    } else if from == Primitive::U8 && to == Primitive::Char {
        Cast::U8ToChar
    } else {
        return None;
    })
}
pub(super) fn integer_type(p: Primitive) -> Option<IntegerType> {
    Some(match p {
        Primitive::I8 => IntegerType::I8,
        Primitive::I16 => IntegerType::I16,
        Primitive::I32 => IntegerType::I32,
        Primitive::I64 => IntegerType::I64,
        Primitive::I128 => IntegerType::I128,
        Primitive::Isize => IntegerType::Isize,
        Primitive::U8 => IntegerType::U8,
        Primitive::U16 => IntegerType::U16,
        Primitive::U32 => IntegerType::U32,
        Primitive::U64 => IntegerType::U64,
        Primitive::U128 => IntegerType::U128,
        Primitive::Usize => IntegerType::Usize,
        _ => return None,
    })
}
pub(super) fn literal(
    value: &Value<'_>,
    ty: Primitive,
    negated: bool,
) -> Result<Scalar, &'static str> {
    match value {
        Value::Integer(integer) => {
            let ty = integer_type(ty).ok_or("integer literal requires an integer type")?;
            if integer.materialize(ty, negated)?.overflow {
                return Err("integer literal is out of range for its type");
            }
            // The operand retains its positive spelling's target bits; unary negation
            // owns the sign, including Rust's signed-minimum literal exception.
            Ok(Scalar::Integer(integer.materialize(ty, false)?.bits))
        }
        Value::Float(float) => {
            let ty = match ty {
                Primitive::F32 => FloatType::F32,
                Primitive::F64 => FloatType::F64,
                _ => return Err("floating literal requires a floating type"),
            };
            let bits = float.round(ty)?;
            let finite = match bits {
                FloatBits::F32(bits) => f32::from_bits(bits).is_finite(),
                FloatBits::F64(bits) => f64::from_bits(bits).is_finite(),
            };
            if !finite {
                return Err("floating literal is out of range for its type");
            }
            Ok(Scalar::Float(bits))
        }
        Value::Boolean(value) if ty == Primitive::Bool => Ok(Scalar::Boolean(*value)),
        Value::Character(value) if ty == Primitive::Char => Ok(Scalar::Character(*value)),
        Value::Byte(value) if ty == Primitive::U8 => Ok(Scalar::Integer(u128::from(*value))),
        _ => Err("invalid scalar literal type"),
    }
}

pub(super) fn compound_operator(op: Op) -> Option<Op> {
    Some(match op {
        Op::PlusEqual => Op::Plus,
        Op::MinusEqual => Op::Minus,
        Op::StarEqual => Op::Star,
        Op::SlashEqual => Op::Slash,
        Op::PercentEqual => Op::Percent,
        Op::AndEqual => Op::And,
        Op::OrEqual => Op::Or,
        Op::CaretEqual => Op::Caret,
        Op::ShiftLeftEqual => Op::ShiftLeft,
        Op::ShiftRightEqual => Op::ShiftRight,
        _ => return None,
    })
}
