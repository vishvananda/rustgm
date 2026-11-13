//! The initial inline assembly profile: the eight option bits and the one
//! table of texts for every form outside the profile. Syntax, native
//! lowering, version-thirteen source export and every version-thirteen reader
//! name a refused form by the same text, in the same order, so they can never
//! disagree about a form or about which of two forms is refused first.
pub const PURE: u16 = 1;
pub const NOMEM: u16 = 2;
pub const READONLY: u16 = 4;
pub const PRESERVES_FLAGS: u16 = 8;
pub const NORETURN: u16 = 16;
pub const NOSTACK: u16 = 32;
pub const ATT_SYNTAX: u16 = 64;
pub const RAW: u16 = 128;
/// Every option bit the syntax defines; no other bit has a meaning.
pub const KNOWN: u16 = 0xff;

/// The defined options outside the profile, reported in bit order before any
/// operand.
pub const OPTIONS: [(u16, &str); 6] = [
    (
        PURE,
        "inline assembly pure option is outside the initial assembly profile",
    ),
    (
        NOMEM,
        "inline assembly nomem option is outside the initial assembly profile",
    ),
    (
        READONLY,
        "inline assembly readonly option is outside the initial assembly profile",
    ),
    (
        NORETURN,
        "inline assembly noreturn option is outside the initial assembly profile",
    ),
    (
        ATT_SYNTAX,
        "inline assembly att_syntax option is outside the initial assembly profile",
    ),
    (
        RAW,
        "inline assembly raw option is outside the initial assembly profile",
    ),
];
pub const OUTPUT: &str = "inline assembly output operand is outside the initial assembly profile";
pub const GENERAL: &str =
    "inline assembly reg input other than a raw pointer is outside the initial assembly profile";
pub const ABCD: &str = "inline assembly reg_abcd operand is outside the initial assembly profile";
pub const BYTE: &str = "inline assembly reg_byte operand is outside the initial assembly profile";
pub const EXPLICIT: &str =
    "inline assembly explicit register operand is outside the initial assembly profile";
pub const XMM: &str = "inline assembly xmm_reg input other than a 128-bit vector is outside the initial assembly profile";
pub const YMM: &str = "inline assembly ymm_reg operand is outside the initial assembly profile";
pub const ZMM: &str = "inline assembly zmm_reg operand is outside the initial assembly profile";
pub const KREG: &str = "inline assembly kreg operand is outside the initial assembly profile";
pub const MODIFIER: &str =
    "inline assembly template modifier is outside the initial assembly profile";
/// `/* */` and `//`, which the pinned backend's assembler lacks: it reads `#`
/// line comments only. Checked last, after the modifiers.
pub const COMMENT: &str =
    "inline assembly comments other than # are outside the initial assembly profile";
/// B1's constant-context text, shared by the local checker and every reader.
pub const CONSTANT: &str = "inline assembly is not permitted in constant contexts";

/// The first defined option outside the profile, in bit order.
///
/// # Errors
/// Returns that option's profile text.
pub fn option(options: u16) -> Result<(), &'static str> {
    match OPTIONS.iter().find(|&&(bit, _)| options & bit != 0) {
        Some(&(_, form)) => Err(form),
        None => Ok(()),
    }
}

/// Where the pinned assembler's reading of an instruction's text stands.
#[derive(Clone, Copy, Default)]
enum Lexeme {
    #[default]
    Code,
    /// Code ending in `/`.
    Slash,
    Quoted,
    /// A quoted backslash, which takes the next byte.
    Escaped,
    /// A `#` comment, which a newline ends.
    Commented,
}
impl Lexeme {
    const fn next(self, byte: u8) -> Result<Self, &'static str> {
        Ok(match (self, byte) {
            (Self::Slash, b'/' | b'*') => return Err(COMMENT),
            (Self::Code | Self::Slash, b'"') => Self::Quoted,
            (Self::Code | Self::Slash, b'#') => Self::Commented,
            (Self::Code, b'/') => Self::Slash,
            (Self::Quoted, b'\\') => Self::Escaped,
            (Self::Quoted, b'"') | (Self::Commented, b'\n') | (Self::Code | Self::Slash, _) => {
                Self::Code
            }
            (Self::Quoted | Self::Escaped, _) => Self::Quoted,
            (Self::Commented, _) => Self::Commented,
        })
    }
    /// A placeholder becomes a register name: ordinary text in any state.
    const fn placeholder(self) -> Self {
        match self {
            Self::Code | Self::Slash => Self::Code,
            Self::Quoted | Self::Escaped => Self::Quoted,
            Self::Commented => Self::Commented,
        }
    }
}
/// One instruction's templates, fed in order, as the pinned assembler reads
/// the text rustc joins with newlines: a double-quoted string with backslash
/// escapes holds no comment, and outside one `#` starts a comment that a
/// newline ends. Any other `/*` or `//` is a comment form outside the
/// profile. The scan is lexical and linear in the template bytes.
#[derive(Default)]
pub struct Comments {
    lexeme: Lexeme,
    joined: bool,
}
impl Comments {
    /// Feed the next template's text around its sorted placeholder spans.
    ///
    /// # Errors
    /// Returns [`COMMENT`] at the first comment form other than `#`, or a
    /// range text when a span leaves its template.
    pub fn template(
        &mut self,
        text: &str,
        pieces: impl IntoIterator<Item = (u32, u32)>,
    ) -> Result<(), &'static str> {
        const OUTSIDE: &str = "assembly placeholder lies outside its template";
        if self.joined {
            self.lexeme = self.lexeme.next(b'\n')?;
        }
        self.joined = true;
        let bytes = text.as_bytes();
        let mut cursor = 0;
        for (start, end) in pieces {
            self.literal(bytes.get(cursor..start as usize).ok_or(OUTSIDE)?)?;
            self.lexeme = self.lexeme.placeholder();
            cursor = end as usize;
        }
        self.literal(bytes.get(cursor..).ok_or(OUTSIDE)?)
    }
    fn literal(&mut self, bytes: &[u8]) -> Result<(), &'static str> {
        for &byte in bytes {
            self.lexeme = self.lexeme.next(byte)?;
        }
        Ok(())
    }
}
