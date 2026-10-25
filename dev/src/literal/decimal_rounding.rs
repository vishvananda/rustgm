//! Exact wide IEEE conversion, adapted from CPPGM's `interchange_float.cpp`.
//! See `shared/CPPGM-NOTICE` and the repository NOTICE for inherited attribution.
//! One format kernel implements nearest/ties-to-even from bounded rational limbs.
mod decimal;
mod unsigned;
use std::cmp::Ordering;
use unsigned::Unsigned;

#[derive(Clone, Copy)]
struct Format {
    precision: u32,
    maximum: i64,
}
impl Format {
    fn infinity(self) -> Result<u128, &'static str> {
        Ok(
            u128::try_from(2 * self.maximum + 1).map_err(|_| "invalid wide format exponent")?
                << (self.precision - 1),
        )
    }
}

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
    rational(&numerator, &denominator, format)
}

fn rational(num: &Unsigned, den: &Unsigned, format: Format) -> Result<u128, &'static str> {
    let length = |value: &Unsigned| {
        i64::try_from(value.bit_len()).map_err(|_| "wide literal bit length overflow")
    };
    let difference = length(num)? - length(den)?;
    let mut scaled = if difference >= 0 {
        den.clone()
    } else {
        num.clone()
    };
    scaled.shift_left(difference.abs())?;
    let at_least = if difference >= 0 {
        num.compare(&scaled) != Ordering::Less
    } else {
        scaled.compare(den) != Ordering::Less
    };
    let exponent = difference - i64::from(!at_least);
    let mut quantum = exponent.max(1 - format.maximum) - i64::from(format.precision - 1);
    let mut numerator = num.clone();
    let mut denominator = den.clone();
    if quantum <= 0 {
        numerator.shift_left(-quantum)?;
    } else {
        denominator.shift_left(quantum)?;
    }
    let divisor = denominator.clone();
    denominator.shift_left(i64::from(format.precision))?;
    let mut significand = 0_u128;
    for bit in (0..=format.precision).rev() {
        if numerator.compare(&denominator) != Ordering::Less {
            numerator.subtract(&denominator);
            significand |= 1_u128 << bit;
        }
        denominator.shift_right_one();
    }
    numerator.shift_left(1)?;
    let half = numerator.compare(&divisor);
    if half == Ordering::Greater || half == Ordering::Equal && significand & 1 != 0 {
        significand += 1;
    }
    if significand & (1_u128 << format.precision) != 0 {
        significand >>= 1;
        quantum += 1;
    }
    let fraction = format.precision - 1;
    let normal = significand & (1_u128 << fraction) != 0;
    let biased = if normal {
        quantum + i64::from(fraction) + format.maximum
    } else {
        0
    };
    if biased > 2 * format.maximum {
        return format.infinity();
    }
    let biased = u128::try_from(biased).map_err(|_| "invalid wide literal exponent")?;
    Ok((significand & ((1_u128 << fraction) - 1)) | (biased << fraction))
}

#[cfg(test)]
#[path = "../../tests/invariants/rounding_scratch.rs"]
mod tests;
