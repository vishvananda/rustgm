//! Literal boundaries and suffixes; values and escape validity belong to PA2.
use super::token::{Kind, LexError};
use super::{Lexer, is_identifier_start};

impl Lexer<'_> {
    /// Raw strings, raw identifiers and prefixed quoted literals. `first` is
    /// `b`, `c` or `r`; any other spelling continues as an identifier.
    pub(super) fn literal_prefix(&mut self, first: u8) -> Result<Option<Kind>, LexError> {
        let second = self.byte(1);
        let raw = match (first, second) {
            (b'b', Some(b'r')) => Some((2, Kind::RawByteString)),
            (b'c', Some(b'r')) => Some((2, Kind::RawCString)),
            (b'r', _) => Some((1, Kind::RawString)),
            _ => None,
        };
        if let Some((prefix, kind)) = raw {
            let tail = &self.text.as_bytes()[self.at as usize + prefix..];
            let hashes = tail.iter().take_while(|byte| **byte == b'#').count();
            if tail.get(hashes) == Some(&b'"') {
                let start = self.at;
                self.skip(prefix);
                return self.raw_string(start, hashes, kind).map(Some);
            }
        }
        if first == b'r' && second == Some(b'#') && self.char_at(2).is_some_and(is_identifier_start)
        {
            self.at += 2;
            self.eat_identifier_continue();
            return Ok(Some(Kind::RawIdentifier));
        }
        let (quote, kind) = match (first, second) {
            (b'b', Some(b'"')) => (b'"', Kind::ByteString),
            (b'c', Some(b'"')) => (b'"', Kind::CString),
            (b'b', Some(b'\'')) => (b'\'', Kind::Byte),
            _ => return Ok(None),
        };
        self.at += 1;
        self.quoted(quote, kind).map(Some)
    }

    fn raw_string(&mut self, start: u32, hashes: usize, kind: Kind) -> Result<Kind, LexError> {
        if hashes > 255 {
            return Err(Self::error(
                start,
                "raw string has more than 255 delimiter hashes",
            ));
        }
        self.skip(hashes + 1);
        let bytes = self.text.as_bytes();
        let mut index = self.at as usize;
        while let Some(offset) = bytes
            .get(index..)
            .and_then(|rest| rest.iter().position(|byte| *byte == b'"'))
        {
            index += offset + 1;
            if bytes
                .get(index..index + hashes)
                .is_some_and(|closing| closing.iter().all(|byte| *byte == b'#'))
            {
                self.skip(index + hashes - self.at as usize);
                self.eat_identifier_continue();
                return Ok(kind);
            }
        }
        self.skip(bytes.len() - self.at as usize);
        Err(Self::error(start, "unterminated raw string"))
    }

    /// Scan bytes to the closing ASCII quote. A backslash skips the next byte;
    /// continuation bytes of a multibyte scalar never match a delimiter.
    pub(super) fn quoted(&mut self, quote: u8, kind: Kind) -> Result<Kind, LexError> {
        let start = self.at;
        let bytes = self.text.as_bytes();
        let mut index = start as usize + 1;
        while let Some(&byte) = bytes.get(index) {
            index += 1;
            if byte == quote {
                self.skip(index - start as usize);
                self.eat_identifier_continue();
                return Ok(kind);
            }
            if byte == b'\\' {
                index = (index + 1).min(bytes.len());
            } else if byte == b'\n' && quote == b'\'' {
                break;
            }
        }
        self.skip(index - start as usize);
        Err(Self::error(start, "unterminated quoted literal"))
    }

    pub(super) fn character_or_lifetime(&mut self) -> Result<Kind, LexError> {
        let start = self.at;
        self.at += 1;
        if self.starts_with(b"r#") && self.char_at(2).is_some_and(is_identifier_start) {
            self.at += 2;
            self.eat_identifier_continue();
            return Ok(Kind::RawLifetime);
        }
        if self
            .char_at(0)
            .is_some_and(|ch| is_identifier_start(ch) || ch.is_ascii_digit())
        {
            self.eat_identifier_continue();
            if self.byte(0) != Some(b'\'') {
                return Ok(Kind::Lifetime);
            }
            self.at += 1;
            self.eat_identifier_continue();
            return Ok(Kind::Character);
        }
        self.at = start;
        self.quoted(b'\'', Kind::Character)
    }

    fn eat_digits(&mut self, hexadecimal: bool) {
        while let Some(byte) = self.byte(0) {
            if !(byte == b'_' || byte.is_ascii_digit() || (hexadecimal && byte.is_ascii_hexdigit()))
            {
                return;
            }
            self.at += 1;
        }
    }

    pub(super) fn number(&mut self) -> Kind {
        let mut kind = Kind::Integer;
        let (decimal, hexadecimal) = match (self.byte(0), self.byte(1)) {
            (Some(b'0'), Some(b'x')) => (false, true),
            (Some(b'0'), Some(b'o' | b'b')) => (false, false),
            _ => (true, false),
        };
        if !decimal {
            self.at += 2;
        }
        self.eat_digits(hexadecimal);
        if decimal {
            if self.byte(0) == Some(b'.')
                && !self
                    .char_at(1)
                    .is_some_and(|ch| ch == '.' || is_identifier_start(ch))
            {
                kind = Kind::Float;
                self.at += 1;
                self.eat_digits(false);
            }
            if matches!(self.byte(0), Some(b'e' | b'E')) {
                kind = Kind::Float;
                self.at += 1;
                if matches!(self.byte(0), Some(b'+' | b'-')) {
                    self.at += 1;
                }
                self.eat_digits(false);
            }
        }
        if self.char_at(0).is_some_and(is_identifier_start) {
            self.eat_identifier_continue();
        }
        kind
    }
}
