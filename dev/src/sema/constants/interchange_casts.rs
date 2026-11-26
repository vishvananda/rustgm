//! Numeric casts share exact significand rounding with wide constant arithmetic.
use super::{arithmetic::mask, interchange_arithmetic::significand, values::Failure};
use crate::{
    literal::value::FloatBits,
    sema::model::{Cast, Scalar},
    support::ieee_rounding::{self, Format},
    types::Primitive,
};

pub(super) fn cast(
    kind: Cast,
    source: Primitive,
    target: Primitive,
    value: Scalar,
) -> Result<Scalar, Failure> {
    match (kind, value) {
        (Cast::IntegerToFloat, Scalar::Integer(value)) => {
            integer_to_float(source, target, value).map(Scalar::Float)
        }
        (Cast::FloatToFloat, Scalar::Float(value)) => {
            float_to_float(source, target, value).map(Scalar::Float)
        }
        (Cast::FloatToInteger, Scalar::Float(value)) => {
            float_to_integer(source, target, value).map(Scalar::Integer)
        }
        _ => Err("unsupported wide constant cast".into()),
    }
}

fn format(ty: Primitive) -> Result<(Format, u128), Failure> {
    let (precision, maximum, sign) = match ty {
        Primitive::F16 => (11, 15, 1 << 15),
        Primitive::F32 => (24, 127, 1 << 31),
        Primitive::F64 => (53, 1023, 1 << 63),
        Primitive::F128 => (113, 16_383, 1 << 127),
        _ => return Err("constant cast requires a floating type".into()),
    };
    Ok((Format { precision, maximum }, sign))
}

fn typed_bits(value: FloatBits, source: Primitive) -> Result<u128, Failure> {
    if !matches!(
        (source, value),
        (Primitive::F16, FloatBits::F16(_))
            | (Primitive::F32, FloatBits::F32(_))
            | (Primitive::F64, FloatBits::F64(_))
            | (Primitive::F128, FloatBits::F128 { .. })
    ) {
        return Err("constant cast value differs from its source width".into());
    }
    Ok(value.bits())
}

fn packed(bits: u128, target: Primitive) -> Result<FloatBits, Failure> {
    Ok(match target {
        Primitive::F16 => {
            FloatBits::F16(u16::try_from(bits).map_err(|_| "invalid half cast bits")?)
        }
        Primitive::F32 => {
            FloatBits::F32(u32::try_from(bits).map_err(|_| "invalid single cast bits")?)
        }
        Primitive::F64 => {
            FloatBits::F64(u64::try_from(bits).map_err(|_| "invalid double cast bits")?)
        }
        Primitive::F128 => FloatBits::from_quad(bits),
        _ => return Err("constant cast requires a floating target".into()),
    })
}

fn integer_to_float(
    source: Primitive,
    target: Primitive,
    value: u128,
) -> Result<FloatBits, Failure> {
    let (signed, width) = source.integer().ok_or("missing integer cast source")?;
    let value = value & mask(width);
    let negative = signed && value & (1 << (width - 1)) != 0;
    let magnitude = if negative {
        0_u128.wrapping_sub(value) & mask(width)
    } else {
        value
    };
    let (target_format, sign) = format(target)?;
    let bits = ieee_rounding::ratio(magnitude, 1, 0, target_format)?;
    packed(bits | if negative { sign } else { 0 }, target)
}

fn float_to_float(
    source: Primitive,
    target: Primitive,
    value: FloatBits,
) -> Result<FloatBits, Failure> {
    let bits = typed_bits(value, source)?;
    if source == target {
        return Ok(value);
    }
    let (source_format, source_sign) = format(source)?;
    let (target_format, target_sign) = format(target)?;
    let sign = if bits & source_sign == 0 {
        0
    } else {
        target_sign
    };
    let magnitude = bits & !source_sign;
    let infinity = source_format.infinity()?;
    let bits = match magnitude.cmp(&infinity) {
        std::cmp::Ordering::Greater => {
            // Rust permits a canonical quiet NaN when converting between widths.
            // Identity casts above preserve all original bits; payload choices are
            // deliberately absent from conversion-result oracles.
            target_format.infinity()? | (1 << (target_format.precision - 2))
        }
        std::cmp::Ordering::Equal => target_format.infinity()?,
        std::cmp::Ordering::Less => {
            let (significand, exponent) = significand(magnitude, source_format)?;
            ieee_rounding::ratio(significand, 1, exponent, target_format)?
        }
    };
    packed(sign | bits, target)
}

fn float_to_integer(
    source: Primitive,
    target: Primitive,
    value: FloatBits,
) -> Result<u128, Failure> {
    let bits = typed_bits(value, source)?;
    let (source_format, sign) = format(source)?;
    let magnitude = bits & !sign;
    let infinity = source_format.infinity()?;
    let (signed, width) = target.integer().ok_or("missing integer cast target")?;
    let negative = bits & sign != 0;
    if magnitude > infinity || negative && !signed {
        return Ok(0);
    }
    let limit = if signed {
        (mask(width) >> 1) + u128::from(negative)
    } else {
        mask(width)
    };
    let value = if magnitude == infinity {
        limit
    } else {
        let (significand, exponent) = significand(magnitude, source_format)?;
        truncated(significand, exponent, limit)
    };
    Ok(if negative {
        0_u128.wrapping_sub(value) & mask(width)
    } else {
        value
    })
}

/// Truncate a finite positive binary value and saturate before any left shift.
fn truncated(significand: u128, exponent: i64, limit: u128) -> u128 {
    let shift = exponent.unsigned_abs();
    if exponent >= 0 {
        if shift >= 128 || significand > limit >> shift {
            limit
        } else {
            significand << shift
        }
    } else if shift >= 128 {
        0
    } else {
        (significand >> shift).min(limit)
    }
}
