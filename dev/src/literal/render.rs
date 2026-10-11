//! Stable PA2 dump. Future syntax/semantics consume Value, not these lines.
use super::{
    interpret,
    value::{FloatBits, Value},
};
use crate::{
    lex::{Lexer, render::RenderError, token::LexError},
    support::quoted,
};
use std::io::{self, Write};

fn value(output: &mut impl Write, literal: &Value<'_>, start: u32, end: u32) -> io::Result<()> {
    match literal {
        Value::Integer(integer) => writeln!(
            output,
            "integer {start} {end} {} {}",
            integer
                .suffix
                .map_or("infer", super::value::IntegerType::name),
            integer.magnitude
        ),
        Value::Float(float) => {
            write!(
                output,
                "float {start} {end} {} ",
                float.suffix.map_or("infer", super::value::FloatType::name)
            )?;
            if let Some(ty) = float.suffix {
                match float.round(ty).map_err(io::Error::other)? {
                    FloatBits::F32(bits) => write!(output, "0x{bits:08x}")?,
                    FloatBits::F64(bits) => write!(output, "0x{bits:016x}")?,
                }
            } else {
                quoted::bytes(output, float.decimal.as_bytes(), false)?;
            }
            writeln!(output)
        }
        Value::Character(ch) => {
            writeln!(output, "character {start} {end} U+{:06X}", u32::from(*ch))
        }
        Value::Byte(byte) => writeln!(output, "byte {start} {end} 0x{byte:02x}"),
        Value::String(bytes) | Value::ByteString(bytes) | Value::CString(bytes) => {
            let (name, nul) = match literal {
                Value::String(_) => ("string", false),
                Value::ByteString(_) => ("byte-string", false),
                _ => ("c-string", true),
            };
            write!(
                output,
                "{name} {start} {end} {} ",
                bytes.len() + usize::from(nul)
            )?;
            quoted::bytes(output, bytes, nul)?;
            writeln!(output)
        }
        Value::Boolean(boolean) => writeln!(output, "boolean {start} {end} {boolean}"),
    }
}

/// # Errors
/// Reports the first bad literal/lexical token or an output failure, retaining
/// only complete output lines preceding an invalid source literal.
pub fn literals(source: &str, output: &mut impl Write) -> Result<(), RenderError> {
    let mut lexer = Lexer::new(source).map_err(RenderError::Lex)?;
    while let Some(token) = lexer.next_token().map_err(RenderError::Lex)? {
        if let Some(literal) = interpret(source, token).map_err(RenderError::Lex)? {
            value(output, &literal, token.start, token.end).map_err(RenderError::Io)?;
        }
    }
    writeln!(output, "eof {}", lexer.offset()).map_err(RenderError::Io)
}

/// Interpret the identical stream without rendering, for throughput measures.
/// Explicit float suffixes still demand their target conversion.
///
/// # Errors
/// Returns the first lexical or literal interpretation error.
pub fn count(source: &str) -> Result<u64, LexError> {
    let mut lexer = Lexer::new(source)?;
    let mut count = 0;
    while let Some(token) = lexer.next_token()? {
        if let Some(literal) = interpret(source, token)? {
            if let Value::Float(float) = literal
                && let Some(ty) = float.suffix
            {
                std::hint::black_box(float.round(ty).map_err(|message| LexError {
                    offset: token.start,
                    message,
                })?);
            }
            count += 1;
        }
    }
    Ok(count)
}
