//! Literal boundaries and suffixes; values and escape validity belong to PA2.
use super::token::{Kind, LexError};
use super::{Lexer, is_identifier_continue, is_identifier_start};

impl Lexer<'_> {
    pub(super) fn literal_prefix(&mut self) -> Result<Option<Kind>, LexError> {
        for (prefix, kind) in [
            ("br", Kind::RawByteString),
            ("cr", Kind::RawCString),
            ("r", Kind::RawString),
        ] {
            if let Some(tail) = self.rest().strip_prefix(prefix) {
                let hashes = tail.bytes().take_while(|byte| *byte == b'#').count();
                if tail.as_bytes().get(hashes) == Some(&b'"') {
                    let start = self.at;
                    for _ in prefix.bytes() {
                        self.bump();
                    }
                    return self.raw_string(start, hashes, kind).map(Some);
                }
            }
        }
        if self.rest().starts_with("r#")
            && self.rest()[2..]
                .chars()
                .next()
                .is_some_and(is_identifier_start)
        {
            self.at += 2;
            self.eat_while(is_identifier_continue);
            return Ok(Some(Kind::RawIdentifier));
        }
        for (prefix, quote, kind) in [
            ("b\"", '"', Kind::ByteString),
            ("c\"", '"', Kind::CString),
            ("b'", '\'', Kind::Byte),
        ] {
            if self.rest().starts_with(prefix) {
                self.bump();
                return self.quoted(quote, kind).map(Some);
            }
        }
        Ok(None)
    }

    fn raw_string(&mut self, start: u32, hashes: usize, kind: Kind) -> Result<Kind, LexError> {
        if hashes > 255 {
            return Err(Self::error(
                start,
                "raw string has more than 255 delimiter hashes",
            ));
        }
        for _ in 0..=hashes {
            self.bump();
        }
        while let Some(ch) = self.peek() {
            self.bump();
            if ch == '"'
                && self
                    .rest()
                    .bytes()
                    .take(hashes)
                    .filter(|b| *b == b'#')
                    .count()
                    == hashes
            {
                for _ in 0..hashes {
                    self.bump();
                }
                self.eat_while(is_identifier_continue);
                return Ok(kind);
            }
        }
        Err(Self::error(start, "unterminated raw string"))
    }

    pub(super) fn quoted(&mut self, quote: char, kind: Kind) -> Result<Kind, LexError> {
        let start = self.at;
        self.bump();
        while let Some(ch) = self.peek() {
            self.bump();
            if ch == quote {
                self.eat_while(is_identifier_continue);
                return Ok(kind);
            }
            if ch == '\\' {
                self.bump();
            } else if ch == '\n' && quote == '\'' {
                break;
            }
        }
        Err(Self::error(start, "unterminated quoted literal"))
    }

    pub(super) fn character_or_lifetime(&mut self) -> Result<Kind, LexError> {
        let start = self.at;
        self.bump();
        if self.rest().starts_with("r#")
            && self.rest()[2..]
                .chars()
                .next()
                .is_some_and(is_identifier_start)
        {
            self.at += 2;
            self.eat_while(is_identifier_continue);
            return Ok(Kind::RawLifetime);
        }
        if self
            .peek()
            .is_some_and(|ch| is_identifier_start(ch) || ch.is_ascii_digit())
        {
            self.eat_while(is_identifier_continue);
            if self.peek() != Some('\'') {
                return Ok(Kind::Lifetime);
            }
            self.bump();
            self.eat_while(is_identifier_continue);
            return Ok(Kind::Character);
        }
        self.at = start;
        self.quoted('\'', Kind::Character)
    }

    pub(super) fn number(&mut self) -> Kind {
        let mut kind = Kind::Integer;
        let mut radix = 10;
        if self.rest().starts_with("0x") {
            radix = 16;
            self.at += 2;
        } else if self.rest().starts_with("0o") || self.rest().starts_with("0b") {
            radix = 2;
            self.at += 2;
        }
        self.eat_while(|ch| {
            ch == '_' || ch.is_ascii_digit() || (radix == 16 && ch.is_ascii_hexdigit())
        });
        if radix == 10 {
            let mut ahead = self.rest().chars();
            if ahead.next() == Some('.')
                && !ahead
                    .next()
                    .is_some_and(|ch| ch == '.' || is_identifier_start(ch))
            {
                kind = Kind::Float;
                self.bump();
                self.eat_while(|ch| ch.is_ascii_digit() || ch == '_');
            }
            if matches!(self.peek(), Some('e' | 'E')) {
                kind = Kind::Float;
                self.bump();
                if matches!(self.peek(), Some('+' | '-')) {
                    self.bump();
                }
                self.eat_while(|ch| ch.is_ascii_digit() || ch == '_');
            }
        }
        if self.peek().is_some_and(is_identifier_start) {
            self.eat_while(is_identifier_continue);
        }
        kind
    }
}
