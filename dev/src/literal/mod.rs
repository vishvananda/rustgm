//! PA2 literal interpretation. Macro tokens are interpreted only on demand.
mod escapes;
mod number;
pub mod render;
mod text;
pub mod value;

use crate::lex::token::{Kind, LexError, Token};
pub use value::Value;

/// Interpret one lexer token without owning its source or consulting context.
/// Non-literal tokens return `None`; punctuation (including unary minus) is
/// deliberately left to the parser. Diagnostics point at the token's start.
///
/// # Errors
/// Rejects invalid spans, malformed literal contents and expression suffixes.
pub fn interpret(source: &str, token: Token) -> Result<Option<Value<'_>>, LexError> {
    let error = |message| LexError {
        offset: token.start,
        message,
    };
    let spelling = source
        .get(token.start as usize..token.end as usize)
        .ok_or_else(|| error("invalid literal span"))?;
    let value = match token.kind {
        Kind::Integer | Kind::Float => {
            let value = number::decode(spelling, token.kind).map_err(error)?;
            if matches!(&value, Value::Integer(integer) if integer.suffix.is_none())
                && matches!(spelling.as_bytes().get(..2), Some(b"0x" | b"0o" | b"0b"))
                && reserved_period(&source[token.end as usize..])
            {
                return Err(error("non-decimal floating literal is not supported"));
            }
            value
        }
        Kind::Character
        | Kind::Byte
        | Kind::String
        | Kind::ByteString
        | Kind::CString
        | Kind::RawString
        | Kind::RawByteString
        | Kind::RawCString => text::decode(spelling, token.kind).map_err(error)?,
        Kind::Identifier if spelling == "true" => Value::Boolean(true),
        Kind::Identifier if spelling == "false" => Value::Boolean(false),
        _ => return Ok(None),
    };
    Ok(Some(value))
}

fn reserved_period(tail: &str) -> bool {
    let mut chars = tail.chars();
    chars.next() == Some('.')
        && !chars
            .next()
            .is_some_and(|ch| ch == '.' || ch == '_' || unicode_ident::is_xid_start(ch))
}
