//! Static format bytecode retains first use of actual input/trait pairs.
#[path = "placeholder_options.rs"]
mod options;
use options::{Characters, Options};
#[path = "named_inputs.rs"]
mod named;
#[cfg(test)]
#[path = "../../../../tests/invariants/encoding_contracts.rs"]
mod tests;
use super::Charge;
use crate::{
    lex::token::LexError,
    support::{library_types::FormatConstructor, retained::Growth},
    syntax::token_tree::error,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Argument {
    pub input: usize,
    pub constructor: FormatConstructor,
}
pub(super) struct Template<'text> {
    pub bytes: Vec<u8>,
    pub arguments: Vec<Argument>,
    pub captured: Vec<&'text str>,
}
#[cfg(test)]
fn compile<'text>(
    text: &'text str,
    count: usize,
    newline: bool,
    charge: &Charge<'_>,
) -> Result<Template<'text>, LexError> {
    compile_named(text, count, &[], false, newline, charge)
}
pub(super) fn compile_named<'text>(
    text: &'text str,
    count: usize,
    names: &[Option<&'text str>],
    allow_capture: bool,
    newline: bool,
    charge: &Charge<'_>,
) -> Result<Template<'text>, LexError> {
    charge(
        count
            .saturating_mul(8)
            .saturating_add(text.len().saturating_mul(8))
            .saturating_add(1),
        0,
    )?;
    if count > usize::from(u16::MAX) {
        return Err(error(
            0,
            "formatting argument count exceeds the supported profile",
        ));
    }
    let mut inputs = named::Inputs::new(count, names, allow_capture, charge)?;
    let capacity = text.len().saturating_mul(4).saturating_add(8);
    let pairs = count
        .saturating_mul(FormatConstructor::FORMAT_COUNT)
        .min(usize::from(u16::MAX));
    let literal_capacity = text.len().saturating_add(usize::from(newline));
    charge(
        0,
        size_of::<Template<'_>>()
            .saturating_add(capacity)
            .saturating_add(literal_capacity)
            .saturating_add(
                count.saturating_mul(size_of::<[Option<u16>; FormatConstructor::FORMAT_COUNT]>()),
            )
            .saturating_add(pairs.saturating_mul(size_of::<Argument>())),
    )?;
    let mut result = Template {
        bytes: Vec::with_capacity(capacity),
        arguments: Vec::with_capacity(pairs),
        captured: Vec::new(),
    };
    let mut positions = vec![[None; FormatConstructor::FORMAT_COUNT]; count];
    let mut literal = String::with_capacity(literal_capacity);
    let mut chars = Characters::new(text);
    let mut implicit = 0;
    let mut next = 0;
    while let Some(character) = chars.next() {
        match character {
            '{' if chars.peek() == Some('{') => {
                chars.next();
                literal.push('{');
            }
            '}' if chars.peek() == Some('}') => {
                chars.next();
                literal.push('}');
            }
            '}' => return Err(error(0, "unmatched closing brace in formatting literal")),
            '{' => {
                append_literal(&literal, &mut result.bytes);
                literal.clear();
                let (input, constructor, options) =
                    placeholder(&mut chars, &mut implicit, &mut inputs)?;
                let position = position(&mut result, &mut positions, input, constructor, charge)?;
                options.emit(position, next, &mut result.bytes);
                next = usize::from(position) + 1;
            }
            character => literal.push(character),
        }
    }
    if newline {
        literal.push('\n');
    }
    append_literal(&literal, &mut result.bytes);
    if positions
        .iter()
        .any(|slots| slots.iter().all(Option::is_none))
    {
        return Err(error(0, "formatting has an unused argument"));
    }
    result.bytes.push(0);
    result.captured = inputs.captured;
    Ok(result)
}
fn placeholder<'text>(
    chars: &mut Characters<'text>,
    implicit: &mut usize,
    inputs: &mut named::Inputs<'text, '_>,
) -> Result<(usize, FormatConstructor, Options), LexError> {
    let mut index = None;
    while let Some(digit) = chars.peek().and_then(|c| c.to_digit(10)) {
        chars.next();
        index = Some(
            index
                .unwrap_or(0_usize)
                .checked_mul(10)
                .and_then(|n| n.checked_add(digit as usize))
                .ok_or_else(|| error(0, "formatting argument index overflow"))?,
        );
    }
    let index = if index.is_none()
        && chars
            .peek()
            .is_some_and(|c| c == '_' || unicode_ident::is_xid_start(c))
    {
        let start = chars.remaining();
        while chars
            .peek()
            .is_some_and(|c| c == '_' || unicode_ident::is_xid_continue(c))
        {
            chars.next();
        }
        inputs.resolve(&start[..start.len() - chars.remaining().len()])?
    } else {
        inputs.positional(index.unwrap_or_else(|| {
            let current = *implicit;
            *implicit += 1;
            current
        }))?
    };
    let (constructor, options) = match chars.next() {
        Some('}') => (FormatConstructor::Display, Options::default()),
        Some(':') => options::parse(chars)?,
        _ => {
            return Err(error(
                0,
                "this formatting option requires a wider source constructor profile",
            ));
        }
    };
    Ok((index, constructor, options))
}
fn position(
    result: &mut Template<'_>,
    positions: &mut Vec<[Option<u16>; FormatConstructor::FORMAT_COUNT]>,
    input: usize,
    constructor: FormatConstructor,
    charge: &Charge<'_>,
) -> Result<u16, LexError> {
    let column = match constructor {
        FormatConstructor::Display => 0,
        FormatConstructor::Debug => 1,
        FormatConstructor::LowerHex => 2,
        FormatConstructor::Arguments => return Err(error(0, "invalid formatting trait selection")),
    };
    if input == positions.len() {
        if input >= usize::from(u16::MAX) {
            return Err(error(0, "too many formatting inputs"));
        }
        let growth = Growth::vector::<[Option<u16>; FormatConstructor::FORMAT_COUNT]>(
            positions.len(),
            positions.capacity(),
            1,
        );
        charge(growth.work.saturating_add(1), growth.bytes)?;
        positions
            .try_reserve(1)
            .map_err(|_| error(0, "formatting pair index allocation failed"))?;
        positions.push([None; FormatConstructor::FORMAT_COUNT]);
    }
    let position = positions
        .get_mut(input)
        .ok_or_else(|| error(0, "formatting argument index is out of range"))?
        .get_mut(column)
        .ok_or_else(|| error(0, "invalid formatting trait slot"))?;
    if let Some(position) = *position {
        return Ok(position);
    }
    if result.arguments.len() >= usize::from(u16::MAX) {
        return Err(error(0, "too many formatting input/trait pairs"));
    }
    let ordinal = u16::try_from(result.arguments.len())
        .map_err(|_| error(0, "too many formatting arguments"))?;
    let growth = Growth::vector::<Argument>(result.arguments.len(), result.arguments.capacity(), 1);
    charge(growth.work.saturating_add(1), growth.bytes)?;
    result
        .arguments
        .try_reserve(1)
        .map_err(|_| error(0, "formatting pair allocation failed"))?;
    result.arguments.push(Argument { input, constructor });
    *position = Some(ordinal);
    Ok(ordinal)
}
fn append_literal(mut text: &str, output: &mut Vec<u8>) {
    while !text.is_empty() {
        let mut end = text.len().min(usize::from(u16::MAX));
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        let length = u16::try_from(end).unwrap_or(u16::MAX);
        if length < 128 {
            output.push(length.to_le_bytes()[0]);
        } else {
            output.push(0x80);
            output.extend_from_slice(&length.to_le_bytes());
        }
        output.extend_from_slice(&text.as_bytes()[..end]);
        text = &text[end..];
    }
}
