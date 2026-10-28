//! Static options are encoded library data; formatting itself stays in source.
use crate::{
    lex::token::LexError, support::library_types::FormatConstructor, syntax::token_tree::error,
};
#[derive(Clone, Copy)]
pub(super) struct Characters<'a> {
    remaining: &'a str,
}
impl<'a> Characters<'a> {
    pub(super) const fn new(text: &'a str) -> Self {
        Self { remaining: text }
    }
    pub(super) const fn remaining(&self) -> &'a str {
        self.remaining
    }
    pub(super) fn peek(&self) -> Option<char> {
        self.remaining.chars().next()
    }
    pub(super) fn next(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.remaining = &self.remaining[character.len_utf8()..];
        Some(character)
    }
}
const DEFAULT_FLAGS: u32 = 0x6000_0020;
#[derive(Clone, Copy)]
pub(super) struct Options {
    flags: u32,
    width: Option<u16>,
    precision: Option<u16>,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            flags: DEFAULT_FLAGS,
            width: None,
            precision: None,
        }
    }
}
impl Options {
    pub(super) fn emit(self, position: u16, next: usize, bytes: &mut Vec<u8>) {
        let flags = self.flags != DEFAULT_FLAGS;
        let width = self.width.filter(|n| *n != 0);
        let precision = self.precision.filter(|n| *n != 0);
        let indexed = next != usize::from(position);
        bytes.push(
            0xc0 | u8::from(flags)
                | u8::from(width.is_some()) << 1
                | u8::from(precision.is_some()) << 2
                | u8::from(indexed) << 3,
        );
        if flags {
            bytes.extend_from_slice(&self.flags.to_le_bytes());
        }
        if let Some(width) = width {
            bytes.extend_from_slice(&width.to_le_bytes());
        }
        if let Some(precision) = precision {
            bytes.extend_from_slice(&precision.to_le_bytes());
        }
        if indexed {
            bytes.extend_from_slice(&position.to_le_bytes());
        }
    }
}
pub(super) fn parse(chars: &mut Characters<'_>) -> Result<(FormatConstructor, Options), LexError> {
    let mut options = Options::default();
    if consume(chars, '}') {
        return Ok((FormatConstructor::Display, options));
    }
    let mut look = *chars;
    let first = look.next();
    let second = look.next();
    if let Some((fill, align)) = first.zip(second.and_then(alignment)) {
        if matches!(fill, '{' | '}') {
            return Err(wider());
        }
        chars.next();
        chars.next();
        options.flags = fill as u32 | align << 29;
    } else if let Some(align) = first.and_then(alignment) {
        chars.next();
        options.flags = u32::from(' ') | align << 29;
    }
    match chars.peek() {
        Some('+') => {
            chars.next();
            options.flags |= 1 << 21;
        }
        Some('-') => {
            chars.next();
            options.flags |= 1 << 22;
        }
        _ => {}
    }
    if consume(chars, '#') {
        options.flags |= 1 << 23;
    }
    if consume(chars, '0') {
        options.flags |= 1 << 24;
    }
    options.width = count(chars)?;
    if options.width.is_some() {
        options.flags |= 1 << 27;
    }
    if consume(chars, '.') {
        options.precision = count(chars)?;
        if options.precision.is_some() {
            options.flags |= 1 << 28;
        }
    }
    let constructor = match chars.next() {
        Some('}') => return Ok((FormatConstructor::Display, options)),
        Some('?') => FormatConstructor::Debug,
        Some('x') if consume(chars, '?') => {
            options.flags |= 1 << 25;
            FormatConstructor::Debug
        }
        Some('X') if consume(chars, '?') => {
            options.flags |= 1 << 26;
            FormatConstructor::Debug
        }
        Some('x') => FormatConstructor::LowerHex,
        _ => return Err(wider()),
    };
    if !consume(chars, '}') {
        return Err(wider());
    }
    Ok((constructor, options))
}
fn count(chars: &mut Characters<'_>) -> Result<Option<u16>, LexError> {
    let mut result = None;
    while let Some(digit) = chars.peek().and_then(|c| c.to_digit(10)) {
        chars.next();
        let digit = u16::try_from(digit).map_err(|_| wider())?;
        result = Some(
            result
                .unwrap_or(0_u16)
                .checked_mul(10)
                .and_then(|n| n.checked_add(digit))
                .ok_or_else(|| error(0, "formatting count exceeds its u16 encoding"))?,
        );
    }
    if chars.peek() == Some('$') {
        return Err(wider());
    }
    Ok(result)
}
fn alignment(character: char) -> Option<u32> {
    match character {
        '<' => Some(0),
        '>' => Some(1),
        '^' => Some(2),
        _ => None,
    }
}
fn consume(chars: &mut Characters<'_>, wanted: char) -> bool {
    if chars.peek() == Some(wanted) {
        chars.next();
        true
    } else {
        false
    }
}
fn wider() -> LexError {
    error(
        0,
        "this formatting option requires a wider source constructor profile",
    )
}
