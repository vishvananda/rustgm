//! One-pass escape decoding. Plain and raw contents stay borrowed.
use std::{borrow::Cow, str::Chars};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Encoding {
    Unicode,
    Bytes,
    CString,
}

#[derive(Clone, Copy)]
pub(super) struct Flavor {
    pub encoding: Encoding,
    pub single: bool,
    pub raw: bool,
}

fn unicode(chars: &mut Chars<'_>) -> Result<char, &'static str> {
    if chars.next() != Some('{') {
        return Err("Unicode escape requires braces");
    }
    let mut value = 0_u32;
    let mut digits = 0;
    for ch in chars.by_ref() {
        if ch == '}' {
            return if digits == 0 {
                Err("Unicode escape has no digits")
            } else {
                char::from_u32(value).ok_or("Unicode escape is not a scalar value")
            };
        }
        if ch == '_' && digits != 0 {
            continue;
        }
        let digit = ch.to_digit(16).ok_or("invalid Unicode escape digit")?;
        digits += 1;
        if digits > 6 {
            return Err("Unicode escape has more than six digits");
        }
        value = value * 16 + digit;
    }
    Err("unterminated Unicode escape")
}

fn hex(chars: &mut Chars<'_>) -> Result<u8, &'static str> {
    let high = chars.next().and_then(|ch| ch.to_digit(16));
    let low = chars.next().and_then(|ch| ch.to_digit(16));
    match (high, low) {
        (Some(high), Some(low)) => u8::try_from(high * 16 + low).map_err(|_| "invalid hex escape"),
        _ => Err("hex escape requires two digits"),
    }
}

enum Escaped {
    Byte(u8),
    Scalar(char),
    Continuation,
}

fn escape(chars: &mut Chars<'_>, flavor: Flavor) -> Result<Escaped, &'static str> {
    let byte = match chars.next().ok_or("incomplete escape")? {
        '0' => 0,
        't' => b'\t',
        'n' => b'\n',
        'r' => b'\r',
        '"' => b'"',
        '\'' => b'\'',
        '\\' => b'\\',
        'x' => {
            let byte = hex(chars)?;
            if flavor.encoding == Encoding::Unicode && !byte.is_ascii() {
                return Err("hex escape exceeds ASCII in Unicode literal");
            }
            byte
        }
        'u' => {
            if flavor.encoding == Encoding::Bytes {
                return Err("Unicode escape in byte literal");
            }
            let scalar = unicode(chars)?;
            if flavor.encoding == Encoding::CString && scalar == '\0' {
                return Err("NUL in C string");
            }
            return Ok(Escaped::Scalar(scalar));
        }
        '\n' if !flavor.single => {
            while chars.as_str().starts_with(['\t', '\n', '\r', ' ']) {
                chars.next();
            }
            return Ok(Escaped::Continuation);
        }
        _ => return Err("invalid escape"),
    };
    if flavor.encoding == Encoding::CString && byte == 0 {
        return Err("NUL in C string");
    }
    Ok(Escaped::Byte(byte))
}

fn validate_bare(ch: char, flavor: Flavor) -> Result<(), &'static str> {
    if ch == '\r' {
        return Err("bare carriage return in literal");
    }
    if flavor.single && matches!(ch, '\t' | '\n' | '\'') {
        return Err("character must be escaped");
    }
    if flavor.encoding == Encoding::Bytes && !ch.is_ascii() {
        return Err("non-ASCII character in byte literal");
    }
    if flavor.encoding == Encoding::CString && ch == '\0' {
        return Err("NUL in C string");
    }
    Ok(())
}

/// Decode a character/byte on the stack; escaped scalar literals need no Vec.
pub(super) fn scalar(content: &str, flavor: Flavor) -> Result<char, &'static str> {
    let length_error = if flavor.encoding == Encoding::Bytes {
        "byte literal must contain exactly one byte"
    } else {
        "character literal must contain exactly one scalar"
    };
    let mut chars = content.chars();
    let first = chars.next().ok_or(length_error)?;
    let scalar = if first == '\\' {
        match escape(&mut chars, flavor)? {
            Escaped::Byte(byte) => char::from(byte),
            Escaped::Scalar(ch) => ch,
            Escaped::Continuation => return Err(length_error),
        }
    } else {
        validate_bare(first, flavor)?;
        first
    };
    if chars.next().is_some() {
        return Err(length_error);
    }
    Ok(scalar)
}

pub(super) fn decode(content: &str, flavor: Flavor) -> Result<Cow<'_, [u8]>, &'static str> {
    // Allocate once, and only if decoding actually changes the byte sequence.
    // Chars' remaining slice supplies offsets without per-character records.
    let mut output: Option<Vec<u8>> = None;
    let mut chars = content.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' && !flavor.raw {
            let prefix_end = content.len() - chars.as_str().len() - 1;
            let buffer = output.get_or_insert_with(|| {
                let mut bytes = Vec::with_capacity(content.len());
                bytes.extend_from_slice(&content.as_bytes()[..prefix_end]);
                bytes
            });
            match escape(&mut chars, flavor)? {
                Escaped::Byte(byte) => buffer.push(byte),
                Escaped::Scalar(ch) => {
                    buffer.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
                }
                Escaped::Continuation => {}
            }
        } else {
            validate_bare(ch, flavor)?;
            if let Some(buffer) = &mut output {
                buffer.extend_from_slice(ch.encode_utf8(&mut [0; 4]).as_bytes());
            }
        }
    }
    Ok(match output {
        Some(buffer) => Cow::Owned(buffer),
        None => Cow::Borrowed(content.as_bytes()),
    })
}
