//! The PA1 text view. The parser will consume Token values, never these lines.
use super::{Lexer, token::LexError};
use crate::support::quoted;
use std::io::{self, Write};

#[derive(Debug)]
pub enum RenderError {
    Lex(LexError),
    Io(io::Error),
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
        quoted::bytes(
            output,
            &text.as_bytes()[token.start as usize..token.end as usize],
            false,
        )
        .map_err(RenderError::Io)?;
        output.write_all(b"\n").map_err(RenderError::Io)?;
    }
    writeln!(output, "eof {}", lexer.offset()).map_err(RenderError::Io)
}
