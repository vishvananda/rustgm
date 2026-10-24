//! Concatenation validates typed eager arguments before allocating its byte result.
#[cfg(test)]
#[path = "../../../tests/invariants/concatenated_bytes.rs"]
mod tests;
use crate::{
    expand::{builtin, config::Config},
    lex::token::LexError,
    literal::{Value, value::IntegerType},
    syntax::{
        ast::{Kind, MacroContext, NodeId, Syntax},
        token_tree::error,
    },
};
use std::borrow::Cow;

enum Part<'a> {
    Byte(u8),
    Bytes(Cow<'a, [u8]>),
    Repeat(u8, Option<usize>),
}
impl Part<'_> {
    fn length(&self) -> Option<usize> {
        match self {
            Self::Byte(_) => Some(1),
            Self::Bytes(bytes) => Some(bytes.len()),
            Self::Repeat(_, count) => *count,
        }
    }
}

pub(crate) fn expand<'a, L>(
    ast: &Syntax<L>,
    root: NodeId,
    mut literal: impl FnMut(u32) -> Result<Value<'a>, LexError>,
    mut charge: impl FnMut(usize, usize) -> Result<(), LexError>,
    grammar: MacroContext,
    config: &Config,
) -> Result<builtin::Expansion, LexError> {
    let mut requested = Some(0_usize);
    walk(ast, root, &mut literal, &mut charge, |part| {
        requested = requested
            .zip(part.length())
            .and_then(|(a, b)| a.checked_add(b));
        Ok(())
    })?;
    // Requested byte copies and their owned literal/diagnostic payload are paid
    // before reserve. Overflow also charges a failed attempt in the real owner.
    let payload = requested
        .and_then(|length| length.checked_mul(32))
        .and_then(|bytes| bytes.checked_add(1024));
    charge(
        requested.unwrap_or(usize::MAX),
        payload.unwrap_or(usize::MAX),
    )?;
    let length = requested.filter(|_| payload.is_some()).ok_or_else(|| {
        error(
            ast.node(root).span.start,
            "concat_bytes output length overflows",
        )
    })?;
    let mut output = Vec::new();
    output.try_reserve_exact(length).map_err(|_| {
        error(
            ast.node(root).span.start,
            "concat_bytes output allocation failed",
        )
    })?;
    // Payload capacity, not malloc bookkeeping. An allocator may round up the
    // exposed capacity; charge its conservative additional payload before fill.
    charge(
        0,
        output.capacity().saturating_sub(length).saturating_mul(32),
    )?;
    walk(ast, root, &mut literal, &mut charge, |part| {
        let end = output
            .len()
            .checked_add(
                part.length()
                    .ok_or_else(|| error(0, "concat_bytes count changed"))?,
            )
            .filter(|end| *end <= length)
            .ok_or_else(|| error(0, "concat_bytes input length changed after preflight"))?;
        match part {
            Part::Byte(byte) => output.push(byte),
            Part::Bytes(bytes) => output.extend_from_slice(&bytes),
            Part::Repeat(byte, _) => output.resize(end, byte),
        }
        Ok(())
    })?;
    if output.len() != length {
        return Err(error(
            0,
            "concat_bytes input length changed after preflight",
        ));
    }
    builtin::byte_string(output, grammar, config)
}

fn walk<'a, L>(
    ast: &Syntax<L>,
    root: NodeId,
    literal: &mut impl FnMut(u32) -> Result<Value<'a>, LexError>,
    charge: &mut impl FnMut(usize, usize) -> Result<(), LexError>,
    mut append: impl FnMut(Part<'a>) -> Result<(), LexError>,
) -> Result<(), LexError> {
    charge(1, 0)?;
    let Kind::Expansion {
        children,
        tail: None,
    } = ast.node(root).kind
    else {
        return Err(error(
            ast.node(root).span.start,
            "concat_bytes requires its actual eager argument list",
        ));
    };
    for &argument in ast.list(children) {
        charge(1, 0)?;
        let actual = ast.node(argument);
        match actual.kind {
            Kind::Array(elements) => {
                for &element in ast.list(elements) {
                    append(Part::Byte(array_byte(ast, element, literal, charge)?))?;
                }
            }
            Kind::Repeat { value, count } => {
                charge(1, 0)?;
                let Kind::Literal(index) = ast.node(count).kind else {
                    return Err(error(
                        ast.node(count).span.start,
                        "concat_bytes repeat count requires an integer literal",
                    ));
                };
                let Value::Integer(integer) = literal(index)? else {
                    return Err(error(
                        ast.node(count).span.start,
                        "concat_bytes repeat count requires an integer literal",
                    ));
                };
                // Even count zero validates the actual repeated element.
                let byte = array_byte(ast, value, literal, charge)?;
                append(Part::Repeat(byte, usize::try_from(integer.magnitude).ok()))?;
            }
            Kind::Literal(index) => match literal(index)? {
                Value::Byte(byte) => append(Part::Byte(byte))?,
                Value::ByteString(bytes) => append(Part::Bytes(bytes))?,
                _ => {
                    return Err(error(
                        actual.span.start,
                        "concat_bytes requires byte or byte-string literals",
                    ));
                }
            },
            _ => {
                return Err(error(
                    actual.span.start,
                    "concat_bytes requires literal arguments",
                ));
            }
        }
    }
    Ok(())
}

fn array_byte<'a, L>(
    ast: &Syntax<L>,
    node: NodeId,
    literal: &mut impl FnMut(u32) -> Result<Value<'a>, LexError>,
    charge: &mut impl FnMut(usize, usize) -> Result<(), LexError>,
) -> Result<u8, LexError> {
    charge(1, 0)?;
    let actual = ast.node(node);
    let index = match actual.kind {
        Kind::Literal(index) => index,
        Kind::Array(_) | Kind::Repeat { .. } => {
            return Err(error(
                actual.span.start,
                "concat_bytes rejects nested arrays",
            ));
        }
        _ => {
            return Err(error(
                actual.span.start,
                "concat_bytes array requires byte literals",
            ));
        }
    };
    match literal(index)? {
        Value::Byte(byte) => Ok(byte),
        Value::Integer(integer) if matches!(integer.suffix, None | Some(IntegerType::U8)) => {
            u8::try_from(integer.magnitude)
                .map_err(|_| error(actual.span.start, "concat_bytes array byte is out of range"))
        }
        Value::ByteString(_) => Err(error(
            actual.span.start,
            "concat_bytes rejects nested byte strings",
        )),
        _ => Err(error(
            actual.span.start,
            "concat_bytes array element must be a byte or u8 integer literal",
        )),
    }
}
