//! Decimal scanning feeds the shared exact IEEE rational rounding kernel.
mod decimal;
use crate::support::ieee_rounding::{Format, Unsigned, round_rational};

pub(super) fn round(text: &str, half: bool) -> Result<u128, &'static str> {
    let format = if half {
        Format {
            precision: 11,
            maximum: 15,
        }
    } else {
        Format {
            precision: 113,
            maximum: 16_383,
        }
    };
    let (digits, exponent, significant) = decimal::scan(text, format)?;
    if digits.is_zero() {
        return Ok(0);
    }
    let order = significant + exponent;
    if (order - 1) * 33_219 >= (format.maximum + 1) * 10_000 {
        return format.infinity();
    }
    if order * 33_220 < (1 - format.maximum - i64::from(format.precision)) * 10_000 {
        return Ok(0);
    }
    let mut numerator = digits;
    let mut denominator = Unsigned::one();
    if exponent >= 0 {
        numerator.power_ten(exponent)?;
    } else {
        denominator.power_ten(-exponent)?;
    }
    round_rational(&numerator, &denominator, format)
}

#[cfg(test)]
#[path = "../../tests/invariants/rounding_scratch.rs"]
mod tests;
