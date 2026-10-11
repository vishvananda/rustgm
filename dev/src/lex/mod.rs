//! Pull lexer: spans borrow one source image; no token or spelling allocation.
pub mod keyword;
mod literals;
mod punctuation;
pub mod render;
pub mod token;

use token::{Kind, LexError, Token};

pub struct Lexer<'source> {
    text: &'source str,
    at: u32,
}

impl<'source> Lexer<'source> {
    /// # Errors
    /// Rejects inputs beyond the compact offset range.
    pub fn new(text: &'source str) -> Result<Self, LexError> {
        if u32::try_from(text.len()).is_err() {
            return Err(LexError {
                offset: 0,
                message: "source exceeds the 32-bit offset limit",
            });
        }
        let mut lexer = Self {
            text,
            at: u32::from(text.starts_with('\u{feff}')) * 3,
        };
        // Rust recognizes a shebang only if the next non-whitespace/comment
        // token after #! is not [. Probe with the same lexer, without copying.
        if lexer.rest().starts_with("#!") && lexer.is_shebang() {
            while lexer.peek().is_some_and(|ch| ch != '\n') {
                lexer.bump();
            }
        }
        Ok(lexer)
    }

    fn is_shebang(&self) -> bool {
        let mut probe = Self {
            text: self.text,
            at: self.at + 2,
        };
        loop {
            match probe.next_token() {
                Ok(Some(Token {
                    kind: Kind::Whitespace | Kind::LineComment | Kind::BlockComment,
                    ..
                })) => {}
                Ok(Some(Token {
                    kind: Kind::Punctuation(token::Punctuation::OpenBracket),
                    ..
                })) => return false,
                _ => return true,
            }
        }
    }

    #[must_use]
    pub const fn offset(&self) -> u32 {
        self.at
    }

    fn rest(&self) -> &'source str {
        &self.text[self.at as usize..]
    }
    fn peek(&self) -> Option<char> {
        self.rest().chars().next()
    }
    fn bump(&mut self) {
        if let Some(ch) = self.peek() {
            // A UTF-8 scalar is at most four bytes; new() bounds total length.
            self.at += match ch.len_utf8() {
                1 => 1,
                2 => 2,
                3 => 3,
                _ => 4,
            };
        }
    }
    fn eat_while(&mut self, predicate: impl Fn(char) -> bool) {
        while self.peek().is_some_and(&predicate) {
            self.bump();
        }
    }
    fn error(start: u32, message: &'static str) -> LexError {
        LexError {
            offset: start,
            message,
        }
    }

    /// Produce one token, retaining no prior tokens. An error ends this scan.
    ///
    /// # Errors
    /// Reports unknown characters and unterminated comments or literals.
    pub fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        let Some(ch) = self.peek() else {
            return Ok(None);
        };
        let start = self.at;
        let kind = if is_whitespace(ch) {
            self.eat_while(is_whitespace);
            Kind::Whitespace
        } else if self.rest().starts_with("//") {
            self.line_comment()
        } else if self.rest().starts_with("/*") {
            self.block_comment()?
        } else if let Some(kind) = self.literal_prefix()? {
            kind
        } else if ch == '\'' {
            self.character_or_lifetime()?
        } else if ch == '"' {
            self.quoted('"', Kind::String)?
        } else if ch.is_ascii_digit() {
            self.number()
        } else if is_identifier_start(ch) {
            self.identifier()
        } else {
            self.punctuation()?
        };
        Ok(Some(Token {
            kind,
            start,
            end: self.at,
        }))
    }

    fn identifier(&mut self) -> Kind {
        self.bump();
        self.eat_while(is_identifier_continue);
        if self.peek().is_some_and(|ch| matches!(ch, '#' | '"' | '\'')) {
            Kind::UnknownPrefix
        } else {
            Kind::Identifier
        }
    }

    fn line_comment(&mut self) -> Kind {
        let kind = if self.rest().starts_with("//!") {
            Kind::InnerLineDoc
        } else if self.rest().starts_with("///") && !self.rest().starts_with("////") {
            Kind::OuterLineDoc
        } else {
            Kind::LineComment
        };
        self.eat_while(|ch| ch != '\n');
        kind
    }

    fn block_comment(&mut self) -> Result<Kind, LexError> {
        let start = self.at;
        let kind = if self.rest().starts_with("/*!") {
            Kind::InnerBlockDoc
        } else if self.rest().starts_with("/**")
            && !self.rest().starts_with("/***")
            && !self.rest().starts_with("/**/")
        {
            Kind::OuterBlockDoc
        } else {
            Kind::BlockComment
        };
        self.at += 2;
        let mut depth = 1_u32;
        while self.peek().is_some() {
            if self.rest().starts_with("/*") {
                self.at += 2;
                depth += 1;
            } else if self.rest().starts_with("*/") {
                self.at += 2;
                depth -= 1;
                if depth == 0 {
                    return Ok(kind);
                }
            } else {
                self.bump();
            }
        }
        Err(Self::error(start, "unterminated block comment"))
    }
}

fn is_whitespace(ch: char) -> bool {
    matches!(
        ch,
        '\t' | '\n'
            | '\u{b}'
            | '\u{c}'
            | '\r'
            | ' '
            | '\u{85}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{2028}'
            | '\u{2029}'
    )
}
fn is_identifier_start(ch: char) -> bool {
    ch == '_' || unicode_ident::is_xid_start(ch)
}
fn is_identifier_continue(ch: char) -> bool {
    unicode_ident::is_xid_continue(ch)
}
