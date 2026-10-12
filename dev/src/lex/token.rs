//! Typed lexical categories. Literal interpretation belongs to PA2.

/// Expansion handles fit the category's existing four-byte slot, keeping all
/// ordinary source tokens at twelve bytes. Overflow is a diagnosed work limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExpansionId([u8; 3]);
impl ExpansionId {
    pub const LIMIT: usize = 1 << 24;
    /// # Errors
    /// Rejects a handle outside the bounded expansion registry.
    pub fn new(value: u32) -> Result<Self, LexError> {
        let [a, b, c, high] = value.to_le_bytes();
        if high != 0 {
            return Err(LexError {
                offset: 0,
                message: "expansion identity exceeds 24 bits",
            });
        }
        Ok(Self([a, b, c]))
    }
    #[must_use]
    pub const fn get(self) -> u32 {
        let [a, b, c] = self.0;
        u32::from_le_bytes([a, b, c, 0])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A parser-only input handle. The source lexer never produces this category.
    Interpolated(ExpansionId),
    /// Definition-site crate identity, introduced only by macro transcription.
    CrateRoot(ExpansionId),
    Whitespace,
    LineComment,
    BlockComment,
    OuterLineDoc,
    InnerLineDoc,
    OuterBlockDoc,
    InnerBlockDoc,
    Identifier,
    RawIdentifier,
    Lifetime,
    RawLifetime,
    Integer,
    Float,
    Character,
    Byte,
    String,
    ByteString,
    CString,
    RawString,
    RawByteString,
    RawCString,
    Punctuation(Punctuation),
    UnknownPrefix,
}

impl Kind {
    /// # Errors
    /// Rejects expansion handle overflow.
    pub fn interpolated(index: u32) -> Result<Self, LexError> {
        ExpansionId::new(index).map(Self::Interpolated)
    }
    /// # Errors
    /// Rejects defining-crate identity overflow.
    pub fn crate_root(index: u32) -> Result<Self, LexError> {
        ExpansionId::new(index).map(Self::CrateRoot)
    }
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Interpolated(_) => "interpolated",
            Self::CrateRoot(_) => "crate-root",
            Self::Whitespace => "whitespace",
            Self::LineComment => "line-comment",
            Self::BlockComment => "block-comment",
            Self::OuterLineDoc => "outer-line-doc",
            Self::InnerLineDoc => "inner-line-doc",
            Self::OuterBlockDoc => "outer-block-doc",
            Self::InnerBlockDoc => "inner-block-doc",
            Self::Identifier => "identifier",
            Self::RawIdentifier => "raw-identifier",
            Self::Lifetime => "lifetime",
            Self::RawLifetime => "raw-lifetime",
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Character => "character",
            Self::Byte => "byte",
            Self::String => "string",
            Self::ByteString => "byte-string",
            Self::CString => "c-string",
            Self::RawString => "raw-string",
            Self::RawByteString => "raw-byte-string",
            Self::RawCString => "raw-c-string",
            Self::Punctuation(_) => "punctuation",
            Self::UnknownPrefix => "unknown-prefix",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Punctuation {
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Caret,
    Bang,
    And,
    Or,
    Equal,
    Less,
    Greater,
    At,
    Underscore,
    Dot,
    Comma,
    Semicolon,
    Colon,
    Pound,
    Dollar,
    Question,
    Tilde,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    AndAnd,
    OrOr,
    ShiftLeft,
    ShiftRight,
    PlusEqual,
    MinusEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,
    CaretEqual,
    AndEqual,
    OrEqual,
    ShiftLeftEqual,
    ShiftRightEqual,
    EqualEqual,
    BangEqual,
    LessEqual,
    GreaterEqual,
    DotDot,
    DotDotDot,
    DotDotEqual,
    ColonColon,
    Arrow,
    FatArrow,
    LeftArrow,
}

/// Spans are byte offsets in the CRLF-normalized file (BOM still occupies bytes).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LexError {
    pub offset: u32,
    pub message: &'static str,
}
