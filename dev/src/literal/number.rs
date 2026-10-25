//! Numeric spelling validation and exact integer magnitudes.
use super::value::{Float, FloatType, Integer, IntegerType, Value};
use crate::lex::token::Kind;
use std::borrow::Cow;

pub(super) fn decode(spelling: &str, kind: Kind) -> Result<Value<'_>, &'static str> {
    let (radix, start) = if spelling.starts_with("0x") {
        (16, 2)
    } else if spelling.starts_with("0o") {
        (8, 2)
    } else if spelling.starts_with("0b") {
        (2, 2)
    } else {
        (10, 0)
    };
    let bytes = spelling.as_bytes();
    let mut end = start;
    while bytes.get(end).is_some_and(|byte| {
        *byte == b'_' || byte.is_ascii_digit() || (radix == 16 && byte.is_ascii_hexdigit())
    }) {
        end += 1;
    }
    if !bytes[start..end].iter().any(u8::is_ascii_hexdigit) {
        return Err("numeric literal has no digits");
    }
    if kind == Kind::Float {
        if bytes.get(end) == Some(&b'.') {
            end += 1;
            while bytes
                .get(end)
                .is_some_and(|b| b.is_ascii_digit() || *b == b'_')
            {
                end += 1;
            }
        }
        if matches!(bytes.get(end), Some(b'e' | b'E')) {
            end += 1;
            if matches!(bytes.get(end), Some(b'+' | b'-')) {
                end += 1;
            }
            let exponent_start = end;
            while bytes
                .get(end)
                .is_some_and(|b| b.is_ascii_digit() || *b == b'_')
            {
                end += 1;
            }
            if !bytes[exponent_start..end].iter().any(u8::is_ascii_digit) {
                return Err("floating exponent has no digits");
            }
        }
    }
    let suffix = &spelling[end..];
    let float_suffix = match suffix {
        "f32" => Some(FloatType::F32),
        "f64" => Some(FloatType::F64),
        "f16" => Some(FloatType::F16),
        "f128" => Some(FloatType::F128),
        _ => None,
    };
    if kind == Kind::Float || float_suffix.is_some() {
        if radix != 10 {
            return Err("non-decimal floating literal is not supported");
        }
        if !suffix.is_empty() && float_suffix.is_none() {
            return Err("invalid floating suffix");
        }
        let body = &spelling[..end];
        let decimal = if body.contains('_') {
            Cow::Owned(body.replace('_', ""))
        } else {
            Cow::Borrowed(body)
        };
        return Ok(Value::Float(Float {
            decimal,
            suffix: float_suffix,
        }));
    }
    let suffix = if suffix.is_empty() {
        None
    } else {
        Some(IntegerType::from_suffix(suffix).ok_or("invalid integer suffix")?)
    };
    let mut magnitude = 0_u128;
    for byte in bytes[start..end].iter().copied().filter(|b| *b != b'_') {
        let digit = char::from(byte)
            .to_digit(radix)
            .ok_or("digit outside literal radix")?;
        magnitude = magnitude
            .checked_mul(u128::from(radix))
            .and_then(|v| v.checked_add(u128::from(digit)))
            .ok_or("integer magnitude exceeds u128")?;
    }
    Ok(Value::Integer(Integer { magnitude, suffix }))
}
