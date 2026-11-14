//! Exact IEEE rounding, adapted from CPPGM's `interchange_float.cpp`.
//! See `shared/CPPGM-NOTICE` and NOTICE for inherited attribution.
//! Literal conversion and constant arithmetic share the bounded rational kernel.
mod unsigned;
use std::cmp::Ordering;
pub(crate) use unsigned::Unsigned;

#[derive(Clone, Copy)]
pub(crate) struct Format {
    pub precision: u32,
    pub maximum: i64,
}
impl Format {
    pub fn infinity(self) -> Result<u128, &'static str> {
        Ok(
            u128::try_from(2 * self.maximum + 1).map_err(|_| "invalid wide format exponent")?
                << (self.precision - 1),
        )
    }
}

/// Round an exact finite binary ratio without a host floating intermediate.
pub(crate) fn ratio(
    numerator: u128,
    denominator: u128,
    exponent: i64,
    format: Format,
) -> Result<u128, &'static str> {
    if denominator == 0 {
        return Err("wide floating ratio has a zero denominator");
    }
    if numerator == 0 {
        return Ok(0);
    }
    scaled(
        &Unsigned::from_u128(numerator),
        &Unsigned::from_u128(denominator),
        exponent,
        format,
    )
}

pub(crate) fn round_rational(
    num: &Unsigned,
    den: &Unsigned,
    format: Format,
) -> Result<u128, &'static str> {
    scaled(num, den, 0, format)
}
fn scaled(
    num: &Unsigned,
    den: &Unsigned,
    scale: i64,
    format: Format,
) -> Result<u128, &'static str> {
    if den.is_zero() {
        return Err("wide floating ratio has a zero denominator");
    }
    if num.is_zero() {
        return Ok(0);
    }
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
    let exponent = (difference - i64::from(!at_least))
        .checked_add(scale)
        .ok_or("wide floating ratio exponent overflow")?;
    if exponent > format.maximum {
        return format.infinity();
    }
    let minimum = 1 - format.maximum - i64::from(format.precision - 1);
    if exponent < minimum - 1 {
        return Ok(0);
    }
    let mut quantum = exponent.max(1 - format.maximum) - i64::from(format.precision - 1);
    let mut numerator = num.clone();
    let mut denominator = den.clone();
    // Keep the binary scale in the output exponent. Finite input significands
    // need only precision-sized shifts, even at opposite exponent extremes.
    let shift = quantum
        .checked_sub(scale)
        .ok_or("wide ratio scale overflow")?;
    if shift <= 0 {
        numerator.shift_left(-shift)?;
    } else {
        denominator.shift_left(shift)?;
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
#[path = "../../tests/invariants/scaled_ratios.rs"]
mod tests;
