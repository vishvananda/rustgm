//! Wide constant division rounds exact target significands, never host floats.
use super::values::Failure;
use crate::{
    literal::value::FloatBits,
    support::ieee_rounding::{self, Format},
    types::Primitive,
};

pub(super) fn divide(
    ty: Primitive,
    left: FloatBits,
    right: FloatBits,
) -> Result<FloatBits, Failure> {
    match (ty, left, right) {
        (Primitive::F16, FloatBits::F16(left), FloatBits::F16(right)) => {
            let bits = divide_bits(
                u128::from(left),
                u128::from(right),
                Format {
                    precision: 11,
                    maximum: 15,
                },
                1 << 15,
            )?;
            Ok(FloatBits::F16(
                u16::try_from(bits).map_err(|_| "invalid half division bits")?,
            ))
        }
        (Primitive::F128, FloatBits::F128 { .. }, FloatBits::F128 { .. }) => {
            Ok(FloatBits::from_quad(divide_bits(
                left.bits(),
                right.bits(),
                Format {
                    precision: 113,
                    maximum: 16_383,
                },
                1 << 127,
            )?))
        }
        _ => Err(Failure::Invalid(
            "mismatched wide constant division operands",
        )),
    }
}
fn divide_bits(left: u128, right: u128, format: Format, sign_bit: u128) -> Result<u128, Failure> {
    let sign = (left ^ right) & sign_bit;
    let left = left & !sign_bit;
    let right = right & !sign_bit;
    let infinity = format.infinity()?;
    if left > infinity
        || right > infinity
        || left == 0 && right == 0
        || left == infinity && right == infinity
    {
        // Rust permits a canonical quiet NaN; neither sign nor input payload is
        // an arithmetic preservation promise. Keep payload choices out of oracles.
        return Ok(infinity | (1 << (format.precision - 2)));
    }
    if left == infinity || right == 0 {
        return Ok(sign | infinity);
    }
    if left == 0 || right == infinity {
        return Ok(sign);
    }
    let (left, a) = significand(left, format)?;
    let (right, b) = significand(right, format)?;
    Ok(sign | ieee_rounding::ratio(left, right, a - b, format)?)
}
fn significand(bits: u128, format: Format) -> Result<(u128, i64), Failure> {
    let fraction = format.precision - 1;
    let exponent =
        i64::try_from(bits >> fraction).map_err(|_| "wide division exponent overflow")?;
    let fraction_bits = bits & ((1 << fraction) - 1);
    let hidden = if exponent == 0 { 0 } else { 1 << fraction };
    Ok((
        fraction_bits | hidden,
        exponent.max(1) - format.maximum - i64::from(fraction),
    ))
}
