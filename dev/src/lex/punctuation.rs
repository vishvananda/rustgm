use super::Lexer;
use super::token::{Kind, LexError, Punctuation as P};

// At most 3 bytes of lookahead; this vocabulary is independent of C++ tokens.
const COMPOUND: &[(&str, P)] = &[
    ("<<=", P::ShiftLeftEqual),
    (">>=", P::ShiftRightEqual),
    ("...", P::DotDotDot),
    ("..=", P::DotDotEqual),
    ("&&", P::AndAnd),
    ("||", P::OrOr),
    ("<<", P::ShiftLeft),
    (">>", P::ShiftRight),
    ("+=", P::PlusEqual),
    ("-=", P::MinusEqual),
    ("*=", P::StarEqual),
    ("/=", P::SlashEqual),
    ("%=", P::PercentEqual),
    ("^=", P::CaretEqual),
    ("&=", P::AndEqual),
    ("|=", P::OrEqual),
    ("==", P::EqualEqual),
    ("!=", P::BangEqual),
    ("<=", P::LessEqual),
    (">=", P::GreaterEqual),
    ("..", P::DotDot),
    ("::", P::ColonColon),
    ("->", P::Arrow),
    ("=>", P::FatArrow),
    ("<-", P::LeftArrow),
];

impl Lexer<'_> {
    pub(super) fn punctuation(&mut self) -> Result<Kind, LexError> {
        for &(spelling, kind) in COMPOUND {
            if self.rest().starts_with(spelling) {
                for _ in spelling.bytes() {
                    self.bump();
                }
                return Ok(Kind::Punctuation(kind));
            }
        }
        let kind = match self.peek() {
            Some('+') => P::Plus,
            Some('-') => P::Minus,
            Some('*') => P::Star,
            Some('/') => P::Slash,
            Some('%') => P::Percent,
            Some('^') => P::Caret,
            Some('!') => P::Bang,
            Some('&') => P::And,
            Some('|') => P::Or,
            Some('=') => P::Equal,
            Some('<') => P::Less,
            Some('>') => P::Greater,
            Some('@') => P::At,
            Some('.') => P::Dot,
            Some(',') => P::Comma,
            Some(';') => P::Semicolon,
            Some(':') => P::Colon,
            Some('#') => P::Pound,
            Some('$') => P::Dollar,
            Some('?') => P::Question,
            Some('~') => P::Tilde,
            Some('(') => P::OpenParen,
            Some(')') => P::CloseParen,
            Some('{') => P::OpenBrace,
            Some('}') => P::CloseBrace,
            Some('[') => P::OpenBracket,
            Some(']') => P::CloseBracket,
            _ => return Err(Self::error(self.at, "unknown token start")),
        };
        self.bump();
        Ok(Kind::Punctuation(kind))
    }
}
