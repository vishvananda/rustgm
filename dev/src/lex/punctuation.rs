use super::Lexer;
use super::token::{Kind, LexError, Punctuation as P};

// At most 3 bytes of lookahead; this vocabulary is independent of C++ tokens.
// The first byte selects the candidates; the longest spelling wins.
impl Lexer<'_> {
    /// Delimiters and other bytes that begin no compound spelling need no lookahead.
    #[inline]
    pub(super) fn punctuation(&mut self, first: u8) -> Result<Kind, LexError> {
        let kind = match first {
            b'(' => P::OpenParen,
            b')' => P::CloseParen,
            b'{' => P::OpenBrace,
            b'}' => P::CloseBrace,
            b'[' => P::OpenBracket,
            b']' => P::CloseBracket,
            b',' => P::Comma,
            b';' => P::Semicolon,
            b'@' => P::At,
            b'#' => P::Pound,
            b'$' => P::Dollar,
            b'?' => P::Question,
            b'~' => P::Tilde,
            _ => return self.operator(first),
        };
        self.at += 1;
        Ok(Kind::Punctuation(kind))
    }

    fn operator(&mut self, first: u8) -> Result<Kind, LexError> {
        let second = self.byte(1);
        let third = self.byte(2);
        let (kind, length) = match (first, second) {
            (b'<', Some(b'<')) if third == Some(b'=') => (P::ShiftLeftEqual, 3),
            (b'>', Some(b'>')) if third == Some(b'=') => (P::ShiftRightEqual, 3),
            (b'.', Some(b'.')) if third == Some(b'.') => (P::DotDotDot, 3),
            (b'.', Some(b'.')) if third == Some(b'=') => (P::DotDotEqual, 3),
            (b'&', Some(b'&')) => (P::AndAnd, 2),
            (b'|', Some(b'|')) => (P::OrOr, 2),
            (b'<', Some(b'<')) => (P::ShiftLeft, 2),
            (b'>', Some(b'>')) => (P::ShiftRight, 2),
            (b'+', Some(b'=')) => (P::PlusEqual, 2),
            (b'-', Some(b'=')) => (P::MinusEqual, 2),
            (b'*', Some(b'=')) => (P::StarEqual, 2),
            (b'/', Some(b'=')) => (P::SlashEqual, 2),
            (b'%', Some(b'=')) => (P::PercentEqual, 2),
            (b'^', Some(b'=')) => (P::CaretEqual, 2),
            (b'&', Some(b'=')) => (P::AndEqual, 2),
            (b'|', Some(b'=')) => (P::OrEqual, 2),
            (b'=', Some(b'=')) => (P::EqualEqual, 2),
            (b'!', Some(b'=')) => (P::BangEqual, 2),
            (b'<', Some(b'=')) => (P::LessEqual, 2),
            (b'>', Some(b'=')) => (P::GreaterEqual, 2),
            (b'.', Some(b'.')) => (P::DotDot, 2),
            (b':', Some(b':')) => (P::ColonColon, 2),
            (b'-', Some(b'>')) => (P::Arrow, 2),
            (b'=', Some(b'>')) => (P::FatArrow, 2),
            (b'<', Some(b'-')) => (P::LeftArrow, 2),
            (b'+', _) => (P::Plus, 1),
            (b'-', _) => (P::Minus, 1),
            (b'*', _) => (P::Star, 1),
            (b'/', _) => (P::Slash, 1),
            (b'%', _) => (P::Percent, 1),
            (b'^', _) => (P::Caret, 1),
            (b'!', _) => (P::Bang, 1),
            (b'&', _) => (P::And, 1),
            (b'|', _) => (P::Or, 1),
            (b'=', _) => (P::Equal, 1),
            (b'<', _) => (P::Less, 1),
            (b'>', _) => (P::Greater, 1),
            (b'.', _) => (P::Dot, 1),
            (b':', _) => (P::Colon, 1),
            _ => return Err(Self::error(self.at, "unknown token start")),
        };
        self.at += length;
        Ok(Kind::Punctuation(kind))
    }
}
