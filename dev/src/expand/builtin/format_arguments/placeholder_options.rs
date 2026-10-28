//! Options encode literal counts or real argument-row indices; formatting stays in source.
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
enum Count<'text> {
    Literal(u16),
    Positional(usize),
    Named(&'text str),
    Input(usize),
}
#[derive(Clone, Copy)]
pub(super) struct Options<'text> {
    flags: u32,
    width: Option<Count<'text>>,
    precision: Option<Count<'text>>,
}
#[derive(Clone, Copy)]
struct EncodedCount {
    value: u16,
    dynamic: bool,
}
pub(super) struct Encoded {
    flags: u32,
    width: Option<EncodedCount>,
    precision: Option<EncodedCount>,
}
impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            flags: DEFAULT_FLAGS,
            width: None,
            precision: None,
        }
    }
}
impl Options<'_> {
    pub(super) fn encoded(
        self,
        mut pair: impl FnMut(usize) -> Result<u16, LexError>,
    ) -> Result<Encoded, LexError> {
        let mut count = |value| match value {
            Count::Literal(value) => Ok(EncodedCount {
                value,
                dynamic: false,
            }),
            Count::Input(input) => Ok(EncodedCount {
                value: pair(input)?,
                dynamic: true,
            }),
            Count::Named(_) | Count::Positional(_) => Err(error(
                0,
                "formatting count was not resolved against actual inputs",
            )),
        };
        // Actual lowering inserts width before precision, after the format pair.
        let width = self.width.map(&mut count).transpose()?;
        let precision = self.precision.map(&mut count).transpose()?;
        Ok(Encoded {
            flags: self.flags,
            width,
            precision,
        })
    }
}
impl<'text> Options<'text> {
    pub(super) fn bind(
        &mut self,
        inputs: &mut super::named::Inputs<'text, '_>,
    ) -> Result<(), LexError> {
        let mut bind = |slot: &mut Option<Count<'text>>| {
            if let Some(value) = slot {
                *value = match *value {
                    Count::Positional(index) => Count::Input(inputs.positional(index)?),
                    Count::Named(name) => Count::Input(inputs.resolve(name)?),
                    value => value,
                };
            }
            Ok(())
        };
        // The builtin's actual capture lookup visits precision before width.
        bind(&mut self.precision)?;
        bind(&mut self.width)
    }
}
impl Encoded {
    pub(super) fn emit(self, position: u16, next: usize, bytes: &mut Vec<u8>) {
        let flags = self.flags != DEFAULT_FLAGS;
        let width = self.width.filter(|n| n.dynamic || n.value != 0);
        let precision = self.precision.filter(|n| n.dynamic || n.value != 0);
        let indexed = next != usize::from(position);
        bytes.push(
            0xc0 | u8::from(flags)
                | u8::from(width.is_some()) << 1
                | u8::from(precision.is_some()) << 2
                | u8::from(indexed) << 3
                | u8::from(width.is_some_and(|n| n.dynamic)) << 4
                | u8::from(precision.is_some_and(|n| n.dynamic)) << 5,
        );
        if flags {
            bytes.extend_from_slice(&self.flags.to_le_bytes());
        }
        if let Some(width) = width {
            bytes.extend_from_slice(&width.value.to_le_bytes());
        }
        if let Some(precision) = precision {
            bytes.extend_from_slice(&precision.value.to_le_bytes());
        }
        if indexed {
            bytes.extend_from_slice(&position.to_le_bytes());
        }
    }
}
pub(super) fn parse<'text>(
    chars: &mut Characters<'text>,
    implicit: &mut usize,
) -> Result<(FormatConstructor, Options<'text>), LexError> {
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
        if consume(chars, '$') {
            options.width = Some(Count::Positional(0));
        } else {
            options.flags |= 1 << 24;
        }
    }
    if options.width.is_none() {
        options.width = count(chars)?;
    }
    if options.width.is_some() {
        options.flags |= 1 << 27;
    }
    if consume(chars, '.') {
        options.precision = if consume(chars, '*') {
            let index = *implicit;
            *implicit = implicit
                .checked_add(1)
                .ok_or_else(|| error(0, "formatting argument index overflow"))?;
            Some(Count::Positional(index))
        } else {
            count(chars)?
        };
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
        Some('X') => FormatConstructor::UpperHex,
        Some('o') => FormatConstructor::Octal,
        Some('b') => FormatConstructor::Binary,
        Some('p') => FormatConstructor::Pointer,
        Some('e') => FormatConstructor::LowerExp,
        Some('E') => FormatConstructor::UpperExp,
        _ => return Err(wider()),
    };
    if !consume(chars, '}') {
        return Err(wider());
    }
    Ok((constructor, options))
}
fn count<'text>(chars: &mut Characters<'text>) -> Result<Option<Count<'text>>, LexError> {
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
    if let Some(value) = result {
        return Ok(Some(if consume(chars, '$') {
            Count::Positional(usize::from(value))
        } else {
            Count::Literal(value)
        }));
    }
    let saved = *chars;
    let start = chars.remaining();
    if chars
        .peek()
        .is_some_and(|c| c == '_' || unicode_ident::is_xid_start(c))
    {
        chars.next();
        while chars
            .peek()
            .is_some_and(|c| c == '_' || unicode_ident::is_xid_continue(c))
        {
            chars.next();
        }
        let name = &start[..start.len() - chars.remaining().len()];
        if name != "_" && consume(chars, '$') {
            return Ok(Some(Count::Named(name)));
        }
    }
    *chars = saved;
    Ok(None)
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
