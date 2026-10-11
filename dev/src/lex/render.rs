//! The PA1 text view. The parser will consume Token values, never these lines.
use super::{Lexer, token::LexError};
use std::io::{self, Write};

#[derive(Debug)]
pub enum RenderError {
    Lex(LexError),
    Io(io::Error),
}

/// Write an ASCII, lossless spelling without an allocation per token.
fn quoted(output: &mut impl Write, spelling: &str) -> io::Result<()> {
    output.write_all(b"\"")?;
    for byte in spelling.bytes() {
        match byte {
            b'"' => output.write_all(b"\\\"")?,
            b'\\' => output.write_all(b"\\\\")?,
            b'\n' => output.write_all(b"\\n")?,
            b'\r' => output.write_all(b"\\r")?,
            b'\t' => output.write_all(b"\\t")?,
            32..=126 => output.write_all(&[byte])?,
            _ => write!(output, "\\x{byte:02x}")?,
        }
    }
    output.write_all(b"\"")
}

/// # Errors
/// Returns lexical errors with the already-written token prefix, or I/O errors.
pub fn tokens(text: &str, output: &mut impl Write) -> Result<(), RenderError> {
    let mut lexer = Lexer::new(text).map_err(RenderError::Lex)?;
    while let Some(token) = lexer.next_token().map_err(RenderError::Lex)? {
        write!(
            output,
            "{} {} {} ",
            token.kind.name(),
            token.start,
            token.end
        )
        .map_err(RenderError::Io)?;
        quoted(output, &text[token.start as usize..token.end as usize]).map_err(RenderError::Io)?;
        output.write_all(b"\n").map_err(RenderError::Io)?;
    }
    writeln!(output, "eof {}", lexer.offset()).map_err(RenderError::Io)
}
