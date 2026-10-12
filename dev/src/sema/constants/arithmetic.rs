//! Target-width constant arithmetic consumes the scalar operations already selected by inference.
use crate::{
    lex::token::Punctuation as Op,
    literal::value::FloatBits,
    sema::model::{Cast, Scalar},
    syntax::ast::Unary,
    types::Primitive,
};

pub(super) fn mask(width: u16) -> u128 {
    u128::MAX >> (128 - width)
}
fn signed(bits: u128, width: u16) -> i128 {
    i128::from_ne_bytes((bits << (128 - width)).to_ne_bytes()) >> (128 - width)
}
fn bits(value: i128) -> u128 {
    u128::from_ne_bytes(value.to_ne_bytes())
}
fn minimum(width: u16) -> i128 {
    signed(1_u128 << (width - 1), width)
}
fn maximum(width: u16) -> i128 {
    i128::from_ne_bytes(((1_u128 << (width - 1)) - 1).to_ne_bytes())
}
pub(super) fn successor(ty: Primitive, value: u128) -> Option<u128> {
    let (signed, width) = ty.integer()?;
    let max = if signed {
        (1_u128 << (width - 1)) - 1
    } else {
        mask(width)
    };
    (value != max).then(|| value.wrapping_add(1) & mask(width))
}
pub(super) fn unary(
    op: Unary,
    ty: Primitive,
    value: Scalar,
    literal: bool,
) -> Result<Scalar, &'static str> {
    Ok(match (op, value) {
        (Unary::Not, Scalar::Boolean(value)) => Scalar::Boolean(!value),
        (Unary::Not, Scalar::Integer(value)) => {
            let (_, width) = ty.integer().ok_or("missing integer operand type")?;
            Scalar::Integer(!value & mask(width))
        }
        (Unary::Negate, Scalar::Integer(value)) => {
            let (is_signed, width) = ty.integer().ok_or("missing integer operand type")?;
            if !is_signed {
                return Err("unsigned constant negation");
            }
            if literal && value == 1_u128 << (width - 1) {
                return Ok(Scalar::Integer(value));
            }
            let value = signed(value, width)
                .checked_neg()
                .filter(|v| *v <= maximum(width))
                .ok_or("constant negation overflows")?;
            Scalar::Integer(bits(value) & mask(width))
        }
        (Unary::Negate, Scalar::Float(FloatBits::F32(value))) => {
            Scalar::Float(FloatBits::F32(value ^ (1 << 31)))
        }
        (Unary::Negate, Scalar::Float(FloatBits::F64(value))) => {
            Scalar::Float(FloatBits::F64(value ^ (1 << 63)))
        }
        _ => return Err("unsupported selected constant unary operation"),
    })
}
fn comparison<T: Copy + PartialEq + PartialOrd>(op: Op, left: T, right: T) -> Option<bool> {
    Some(match op {
        Op::EqualEqual => left == right,
        Op::BangEqual => left != right,
        Op::Less => left < right,
        Op::LessEqual => left <= right,
        Op::Greater => left > right,
        Op::GreaterEqual => left >= right,
        _ => return None,
    })
}
fn numeric<T>(op: Op, left: T, right: T) -> Result<T, &'static str>
where
    T: std::ops::Add<Output = T>
        + std::ops::Sub<Output = T>
        + std::ops::Mul<Output = T>
        + std::ops::Div<Output = T>
        + std::ops::Rem<Output = T>,
{
    Ok(match op {
        Op::Plus => left + right,
        Op::Minus => left - right,
        Op::Star => left * right,
        Op::Slash => left / right,
        Op::Percent => left % right,
        _ => return Err("unsupported selected floating constant operation"),
    })
}
pub(super) fn binary(
    op: Op,
    ty: Primitive,
    left: Scalar,
    right: Scalar,
) -> Result<Scalar, &'static str> {
    match (left, right) {
        (Scalar::Integer(left), Scalar::Integer(right)) => integer(op, ty, left, right),
        (Scalar::Boolean(left), Scalar::Boolean(right)) => {
            let value = comparison(op, left, right)
                .or(match op {
                    Op::And | Op::AndAnd => Some(left & right),
                    Op::Or | Op::OrOr => Some(left | right),
                    Op::Caret => Some(left ^ right),
                    _ => None,
                })
                .ok_or("unsupported selected boolean constant operation")?;
            Ok(Scalar::Boolean(value))
        }
        (Scalar::Character(left), Scalar::Character(right)) => comparison(op, left, right)
            .map(Scalar::Boolean)
            .ok_or("unsupported selected character constant operation"),
        (Scalar::Float(FloatBits::F32(left)), Scalar::Float(FloatBits::F32(right))) => {
            let (left, right) = (f32::from_bits(left), f32::from_bits(right));
            if let Some(value) = comparison(op, left, right) {
                return Ok(Scalar::Boolean(value));
            }
            Ok(Scalar::Float(FloatBits::F32(
                numeric(op, left, right)?.to_bits(),
            )))
        }
        (Scalar::Float(FloatBits::F64(left)), Scalar::Float(FloatBits::F64(right))) => {
            let (left, right) = (f64::from_bits(left), f64::from_bits(right));
            if let Some(value) = comparison(op, left, right) {
                return Ok(Scalar::Boolean(value));
            }
            Ok(Scalar::Float(FloatBits::F64(
                numeric(op, left, right)?.to_bits(),
            )))
        }
        _ => Err("mismatched selected constant operands"),
    }
}
fn integer(op: Op, ty: Primitive, left: u128, right: u128) -> Result<Scalar, &'static str> {
    let (is_signed, width) = ty.integer().ok_or("missing constant integer type")?;
    let comparison = if is_signed {
        comparison(op, signed(left, width), signed(right, width))
    } else {
        comparison(op, left, right)
    };
    if let Some(value) = comparison {
        return Ok(Scalar::Boolean(value));
    }
    let value = match op {
        Op::And => left & right,
        Op::Or => left | right,
        Op::Caret => left ^ right,
        Op::ShiftLeft | Op::ShiftRight => {
            // A negative signed RHS has a high target-width bit and fails this
            // range check too. The RHS may have a different integer type.
            if right >= u128::from(width) {
                return Err("constant shift amount is out of range");
            }
            let amount = u32::try_from(right).map_err(|_| "invalid shift amount")?;
            match op {
                Op::ShiftLeft => left << amount,
                _ if is_signed => bits(signed(left, width) >> amount),
                _ => left >> amount,
            }
        }
        _ if is_signed => {
            let (left, right) = (signed(left, width), signed(right, width));
            let value = match op {
                Op::Plus => left.checked_add(right),
                Op::Minus => left.checked_sub(right),
                Op::Star => left.checked_mul(right),
                Op::Slash => left.checked_div(right),
                Op::Percent => left.checked_rem(right),
                _ => return Err("unsupported selected integer constant operation"),
            }
            .filter(|v| *v >= minimum(width) && *v <= maximum(width))
            .ok_or("constant integer arithmetic overflows or divides by zero")?;
            // Narrow signed MIN / -1 and MIN % -1 both overflow in Rust.
            if matches!(op, Op::Slash | Op::Percent) && left == minimum(width) && right == -1 {
                return Err("constant signed division overflows");
            }
            bits(value)
        }
        _ => match op {
            Op::Plus => left.checked_add(right),
            Op::Minus => left.checked_sub(right),
            Op::Star => left.checked_mul(right),
            Op::Slash => left.checked_div(right),
            Op::Percent => left.checked_rem(right),
            _ => return Err("unsupported selected integer constant operation"),
        }
        .filter(|v| *v <= mask(width))
        .ok_or("constant integer arithmetic overflows or divides by zero")?,
    };
    Ok(Scalar::Integer(value & mask(width)))
}
#[expect(
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    reason = "These conversions implement Rust numeric casts: IEEE rounding and saturating float-to-integer conversion"
)]
pub(super) fn cast(
    kind: Cast,
    source: Primitive,
    target: Primitive,
    value: Scalar,
) -> Result<Scalar, &'static str> {
    match (kind, value) {
        (Cast::IntegerToInteger, Scalar::Integer(value)) => {
            let (is_signed, width) = source.integer().ok_or("missing cast source type")?;
            let (_, target_width) = target.integer().ok_or("missing cast target type")?;
            Ok(Scalar::Integer(
                (if is_signed {
                    bits(signed(value, width))
                } else {
                    value
                }) & mask(target_width),
            ))
        }
        (Cast::BoolToInteger, Scalar::Boolean(value)) => Ok(Scalar::Integer(u128::from(value))),
        (Cast::CharToInteger, Scalar::Character(value)) => {
            let (_, width) = target.integer().ok_or("missing character cast target")?;
            Ok(Scalar::Integer(u128::from(u32::from(value)) & mask(width)))
        }
        (Cast::U8ToChar, Scalar::Integer(value)) => Ok(Scalar::Character(char::from(
            u8::try_from(value).map_err(|_| "invalid u8 character cast")?,
        ))),
        (Cast::IntegerToFloat, Scalar::Integer(value)) => {
            let (is_signed, width) = source.integer().ok_or("missing floating cast source")?;
            Ok(Scalar::Float(match target {
                Primitive::F32 => FloatBits::F32(
                    (if is_signed {
                        signed(value, width) as f32
                    } else {
                        value as f32
                    })
                    .to_bits(),
                ),
                Primitive::F64 => FloatBits::F64(
                    (if is_signed {
                        signed(value, width) as f64
                    } else {
                        value as f64
                    })
                    .to_bits(),
                ),
                _ => return Err("invalid floating cast target"),
            }))
        }
        (Cast::FloatToInteger, Scalar::Float(value)) => {
            let value = match value {
                FloatBits::F32(v) => f64::from(f32::from_bits(v)),
                FloatBits::F64(v) => f64::from_bits(v),
            };
            let (is_signed, width) = target.integer().ok_or("missing floating cast target")?;
            Ok(Scalar::Integer(if is_signed {
                bits((value as i128).clamp(minimum(width), maximum(width))) & mask(width)
            } else {
                (value as u128).min(mask(width))
            }))
        }
        (Cast::FloatToFloat, Scalar::Float(value)) => Ok(Scalar::Float(match (value, target) {
            (FloatBits::F32(v), Primitive::F64) => {
                FloatBits::F64(f64::from(f32::from_bits(v)).to_bits())
            }
            (FloatBits::F64(v), Primitive::F32) => {
                FloatBits::F32((f64::from_bits(v) as f32).to_bits())
            }
            (value, _) => value,
        })),
        _ => Err("unsupported selected constant cast"),
    }
}
