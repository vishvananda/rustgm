//! Value digits of an already validated decimal, with a bounded sticky tail.
use super::{Format, Unsigned};

pub(super) fn scan(text: &str, format: Format) -> Result<(Unsigned, i64, i64), &'static str> {
    let precision = i64::from(format.precision);
    let kept = (precision * 30_103 + (format.maximum - 1 + precision) * 69_897) / 100_000 + 3;
    let mut digits = Unsigned::default();
    let mut exponent = 0_i64;
    let mut significant = 0_i64;
    let mut point = false;
    let mut seen_digit = false;
    let mut sticky = false;
    let mut chunk = 0_u32;
    let mut count = 0;
    let mut scale = 1_u32;
    let mut bytes = text.bytes().peekable();
    while let Some(byte) = bytes.next_if(|byte| byte.is_ascii_digit() || *byte == b'.') {
        if byte == b'.' {
            if point {
                return Err("invalid validated wide decimal point");
            }
            point = true;
            continue;
        }
        seen_digit = true;
        let value = u32::from(byte - b'0');
        if significant >= kept {
            sticky |= value != 0;
            exponent = exponent.saturating_add(i64::from(!point));
            continue;
        }
        exponent -= i64::from(point);
        if significant == 0 && value == 0 {
            continue;
        }
        chunk = chunk * 10 + value;
        scale *= 10;
        significant += 1;
        count += 1;
        if count == 9 {
            digits.multiply_add(scale, chunk)?;
            (chunk, count, scale) = (0, 0, 1);
        }
    }
    if !seen_digit {
        return Err("wide decimal has no digits");
    }
    if count != 0 {
        digits.multiply_add(scale, chunk)?;
    }
    if sticky {
        digits.multiply_add(10, 1)?;
        exponent -= 1;
        significant += 1;
    }
    if bytes.next_if(|byte| matches!(byte, b'e' | b'E')).is_some() {
        let negative = bytes.next_if(|byte| matches!(byte, b'+' | b'-')) == Some(b'-');
        let mut written = 0_i64;
        if bytes.peek().is_none() {
            return Err("wide decimal exponent has no digits");
        }
        for byte in bytes {
            if !byte.is_ascii_digit() {
                return Err("invalid validated wide decimal exponent");
            }
            written = (written * 10 + i64::from(byte - b'0')).min(1_000_000_000);
        }
        exponent = exponent.saturating_add(if negative { -written } else { written });
    } else if bytes.next().is_some() {
        return Err("invalid validated wide decimal tail");
    }
    Ok((digits, exponent, significant))
}
