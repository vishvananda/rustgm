//! The pinned default-Debug template bytecode, with stable first-use argument order.
#[cfg(test)]
#[path = "../../../../tests/invariants/encoding_contracts.rs"]
mod tests;
use super::Charge;
use crate::{lex::token::LexError, syntax::token_tree::error};

pub(super) struct Template {
    pub bytes: Vec<u8>,
    pub arguments: Vec<usize>,
}
pub(super) fn compile(
    text: &str,
    count: usize,
    newline: bool,
    charge: &Charge<'_>,
) -> Result<Template, LexError> {
    charge(count.saturating_add(text.len()).saturating_add(1), 0)?;
    if count == 0 || count > usize::from(u16::MAX) {
        return Err(error(
            0,
            "formatting argument count exceeds the supported profile",
        ));
    }
    let capacity = text.len().saturating_mul(4).saturating_add(8);
    charge(
        0,
        capacity
            .saturating_add(text.len())
            .saturating_add(count.saturating_mul(size_of::<Option<u16>>() + size_of::<usize>())),
    )?;
    let mut result = Template {
        bytes: Vec::with_capacity(capacity),
        arguments: Vec::with_capacity(count),
    };
    let mut positions = vec![None; count];
    let mut literal = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut implicit = 0;
    let mut next = 0;
    while let Some(character) = chars.next() {
        match character {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                literal.push('{');
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                literal.push('}');
            }
            '}' => return Err(error(0, "unmatched closing brace in formatting literal")),
            '{' => {
                append_literal(&literal, &mut result.bytes);
                literal.clear();
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
                let index = index.unwrap_or_else(|| {
                    let current = implicit;
                    implicit += 1;
                    current
                });
                if chars.next() != Some(':')
                    || chars.next() != Some('?')
                    || chars.next() != Some('}')
                {
                    return Err(error(
                        0,
                        "this formatting option requires a wider source constructor profile",
                    ));
                }
                let position = positions
                    .get_mut(index)
                    .ok_or_else(|| error(0, "formatting argument index is out of range"))?;
                let position = if let Some(position) = *position {
                    position
                } else {
                    let ordinal = u16::try_from(result.arguments.len())
                        .map_err(|_| error(0, "too many formatting arguments"))?;
                    result.arguments.push(index);
                    *position = Some(ordinal);
                    ordinal
                };
                result.bytes.push(if next == usize::from(position) {
                    0xc0
                } else {
                    0xc8
                });
                if next != usize::from(position) {
                    result.bytes.extend_from_slice(&position.to_le_bytes());
                }
                next = usize::from(position) + 1;
            }
            character => literal.push(character),
        }
    }
    if newline {
        literal.push('\n');
    }
    append_literal(&literal, &mut result.bytes);
    if positions.iter().any(Option::is_none) {
        return Err(error(0, "formatting has an unused argument"));
    }
    result.bytes.push(0);
    Ok(result)
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
