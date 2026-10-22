//! Pull lexer: spans borrow one source image; no token or spelling allocation.
//! Dispatch reads bytes; only non-ASCII positions decode a scalar value.
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
        Self::source(text, true)
    }

    pub(crate) fn source(text: &'source str, strip_shebang: bool) -> Result<Self, LexError> {
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
        if strip_shebang && lexer.starts_with(b"#!") && lexer.is_shebang() {
            lexer.skip_line();
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

    /// The byte `ahead` positions past the current offset.
    fn byte(&self, ahead: usize) -> Option<u8> {
        self.text.as_bytes().get(self.at as usize + ahead).copied()
    }
    fn starts_with(&self, prefix: &[u8]) -> bool {
        self.text
            .as_bytes()
            .get(self.at as usize..)
            .is_some_and(|rest| rest.starts_with(prefix))
    }
    /// The scalar value starting `ahead` bytes on; ASCII needs no decoding.
    fn char_at(&self, ahead: usize) -> Option<char> {
        let index = self.at as usize + ahead;
        let byte = *self.text.as_bytes().get(index)?;
        if byte.is_ascii() {
            return Some(char::from(byte));
        }
        self.text.get(index..)?.chars().next()
    }
    /// Advance by `count` bytes; callers only skip whole scalar values.
    fn skip(&mut self, count: usize) {
        // new() bounds the total length, so every in-text offset fits u32.
        self.at += u32::try_from(count).unwrap_or(u32::MAX - self.at);
    }
    /// Advance past `ch`, which starts at the current offset.
    fn skip_char(&mut self, ch: char) {
        self.skip(ch.len_utf8());
    }
    /// Advance to the next line feed, or to the end of input.
    fn skip_line(&mut self) {
        let rest = &self.text.as_bytes()[self.at as usize..];
        self.skip(
            rest.iter()
                .position(|byte| *byte == b'\n')
                .unwrap_or(rest.len()),
        );
    }
    /// Consume identifier-continue scalars, scanning ASCII bytes by table.
    fn eat_identifier_continue(&mut self) {
        let bytes = self.text.as_bytes();
        let start = self.at as usize;
        let mut index = start;
        while let Some(&byte) = bytes.get(index) {
            if CLASS[usize::from(byte)] & IDENTIFIER != 0 {
                index += 1;
            } else if byte.is_ascii() {
                break;
            } else {
                match self.text.get(index..).and_then(|rest| rest.chars().next()) {
                    Some(ch) if is_identifier_continue(ch) => index += ch.len_utf8(),
                    _ => break,
                }
            }
        }
        self.skip(index - start);
    }
    /// Consume one maximal whitespace run, which is exactly one whitespace token.
    fn eat_whitespace(&mut self) {
        let bytes = self.text.as_bytes();
        let start = self.at as usize;
        let mut index = start;
        while let Some(&byte) = bytes.get(index) {
            if CLASS[usize::from(byte)] & SPACE != 0 {
                index += 1;
            } else if byte.is_ascii() {
                break;
            } else {
                match self.text.get(index..).and_then(|rest| rest.chars().next()) {
                    Some(ch) if is_whitespace(ch) => index += ch.len_utf8(),
                    _ => break,
                }
            }
        }
        self.skip(index - start);
    }
    fn error(start: u32, message: &'static str) -> LexError {
        LexError {
            offset: start,
            message,
        }
    }

    /// The next token other than whitespace or an ordinary comment. `produced`
    /// counts every token consumed, including skipped trivia and the result.
    ///
    /// # Errors
    /// Reports the same lexical failures, in the same order, as `next_token`.
    pub(crate) fn next_significant(
        &mut self,
        produced: &mut usize,
    ) -> Result<Option<Token>, LexError> {
        loop {
            if self
                .byte(0)
                .is_some_and(|byte| CLASS[usize::from(byte)] & SPACE != 0 || !byte.is_ascii())
            {
                let before = self.at;
                self.eat_whitespace();
                if self.at != before {
                    *produced += 1;
                }
            }
            let Some(token) = self.next_token()? else {
                return Ok(None);
            };
            *produced += 1;
            if !matches!(
                token.kind,
                Kind::Whitespace | Kind::LineComment | Kind::BlockComment
            ) {
                return Ok(Some(token));
            }
        }
    }

    /// Produce one token, retaining no prior tokens. An error ends this scan.
    ///
    /// # Errors
    /// Reports unknown characters and unterminated comments or literals.
    pub fn next_token(&mut self) -> Result<Option<Token>, LexError> {
        let Some(first) = self.byte(0) else {
            return Ok(None);
        };
        let start = self.at;
        let kind = match first {
            b'\t' | b'\n' | 0x0b | 0x0c | b'\r' | b' ' => {
                self.at += 1;
                self.eat_whitespace();
                Kind::Whitespace
            }
            b'/' => match self.byte(1) {
                Some(b'/') => self.line_comment(),
                Some(b'*') => self.block_comment()?,
                _ => self.punctuation(first)?,
            },
            b'b' | b'c' | b'r' => match self.literal_prefix(first)? {
                Some(kind) => kind,
                None => self.identifier(1),
            },
            b'\'' => self.character_or_lifetime()?,
            b'"' => self.quoted(b'"', Kind::String)?,
            b'0'..=b'9' => self.number(),
            b'a'..=b'z' | b'A'..=b'Z' | b'_' => self.identifier(1),
            0x80.. => self.non_ascii()?,
            _ => self.punctuation(first)?,
        };
        Ok(Some(Token {
            kind,
            start,
            end: self.at,
        }))
    }

    /// Unicode whitespace, a Unicode identifier, or an unknown token start.
    fn non_ascii(&mut self) -> Result<Kind, LexError> {
        match self.char_at(0) {
            Some(ch) if is_whitespace(ch) => {
                self.skip_char(ch);
                self.eat_whitespace();
                Ok(Kind::Whitespace)
            }
            Some(ch) if is_identifier_start(ch) => Ok(self.identifier(ch.len_utf8())),
            _ => Err(Self::error(self.at, "unknown token start")),
        }
    }

    /// `first` is the byte length of the already classified starting scalar.
    /// ASCII continuation scans inline; a non-ASCII byte takes the decoding scan.
    // Forced only without debug assertions: debug builds keep the call so
    // recursive-descent frames stay small enough for the nesting limit.
    #[cfg_attr(not(debug_assertions), inline(always))]
    #[cfg_attr(
        not(debug_assertions),
        expect(
            clippy::inline_always,
            reason = "per-identifier ASCII scan; callgrind shows calls dominate"
        )
    )]
    fn identifier(&mut self, first: usize) -> Kind {
        let bytes = self.text.as_bytes();
        let start = self.at as usize;
        let mut index = start + first;
        while let Some(&byte) = bytes.get(index)
            && CLASS[usize::from(byte)] & IDENTIFIER != 0
        {
            index += 1;
        }
        self.skip(index - start);
        if bytes.get(index).is_some_and(|byte| !byte.is_ascii()) {
            self.eat_identifier_continue();
        }
        if matches!(self.byte(0), Some(b'#' | b'"' | b'\'')) {
            Kind::UnknownPrefix
        } else {
            Kind::Identifier
        }
    }

    fn line_comment(&mut self) -> Kind {
        let kind = match (self.byte(2), self.byte(3)) {
            (Some(b'!'), _) => Kind::InnerLineDoc,
            (Some(b'/'), next) if next != Some(b'/') => Kind::OuterLineDoc,
            _ => Kind::LineComment,
        };
        self.skip_line();
        kind
    }

    fn block_comment(&mut self) -> Result<Kind, LexError> {
        let start = self.at;
        let kind = match (self.byte(2), self.byte(3)) {
            (Some(b'!'), _) => Kind::InnerBlockDoc,
            (Some(b'*'), next) if !matches!(next, Some(b'*' | b'/')) => Kind::OuterBlockDoc,
            _ => Kind::BlockComment,
        };
        let bytes = self.text.as_bytes();
        let mut index = start as usize + 2;
        let mut depth = 1_u32;
        while let Some(&byte) = bytes.get(index) {
            let next = bytes.get(index + 1).copied();
            if byte == b'/' && next == Some(b'*') {
                index += 2;
                depth += 1;
            } else if byte == b'*' && next == Some(b'/') {
                index += 2;
                depth -= 1;
                if depth == 0 {
                    self.skip(index - start as usize);
                    return Ok(kind);
                }
            } else {
                index += 1;
            }
        }
        self.skip(bytes.len() - start as usize);
        Err(Self::error(start, "unterminated block comment"))
    }
}

const IDENTIFIER: u8 = 1;
const SPACE: u8 = 2;
/// ASCII byte classes: identifier-continue letters, digits and `_`, and the
/// ASCII members of Rust's pattern whitespace. Non-ASCII bytes have no class.
const CLASS: [u8; 256] = {
    let mut table = [0_u8; 256];
    let mut ch = 0_u8;
    while ch < 128 {
        if ch.is_ascii_alphanumeric() || ch == b'_' {
            table[ch as usize] = IDENTIFIER;
        } else if matches!(ch, b'\t' | b'\n' | 0x0b | 0x0c | b'\r' | b' ') {
            table[ch as usize] = SPACE;
        }
        ch += 1;
    }
    table
};

pub(crate) fn is_whitespace(ch: char) -> bool {
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
