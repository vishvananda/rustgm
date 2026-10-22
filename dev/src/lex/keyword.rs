//! Edition-2024 strict and reserved words. Weak words remain contextual.
use super::token::Kind;

/// Identifier spellings the grammar distinguishes. The cursor classifies each
/// significant identifier once, so keyword probes compare small integers.
/// Every strict or reserved word precedes `Union`; the weak words from `Union`
/// onward remain ordinary identifiers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Word {
    Underscore,
    As,
    Async,
    Await,
    Break,
    Const,
    Continue,
    Crate,
    Dyn,
    Else,
    Enum,
    Extern,
    False,
    Fn,
    For,
    If,
    Impl,
    In,
    Let,
    Loop,
    Match,
    Mod,
    Move,
    Mut,
    Pub,
    Ref,
    Return,
    SelfValue,
    SelfType,
    Static,
    Struct,
    Super,
    Trait,
    True,
    Type,
    Unsafe,
    Use,
    Where,
    While,
    Abstract,
    Become,
    Box,
    Do,
    Final,
    Gen,
    Macro,
    Override,
    Priv,
    Try,
    Typeof,
    Unsized,
    Virtual,
    Yield,
    Union,
    MacroRules,
    Safe,
    Raw,
    /// Any other spelling, and every non-identifier token.
    Other,
}

impl Word {
    /// Classify one identifier spelling with a length switch and short compares.
    #[must_use]
    pub fn classify(name: &[u8]) -> Self {
        match name.len() {
            1 if name == b"_" => Self::Underscore,
            2 => match name {
                b"as" => Self::As,
                b"do" => Self::Do,
                b"fn" => Self::Fn,
                b"if" => Self::If,
                b"in" => Self::In,
                _ => Self::Other,
            },
            3 => match name {
                b"box" => Self::Box,
                b"dyn" => Self::Dyn,
                b"for" => Self::For,
                b"gen" => Self::Gen,
                b"let" => Self::Let,
                b"mod" => Self::Mod,
                b"mut" => Self::Mut,
                b"pub" => Self::Pub,
                b"raw" => Self::Raw,
                b"ref" => Self::Ref,
                b"try" => Self::Try,
                b"use" => Self::Use,
                _ => Self::Other,
            },
            4 => match name {
                b"else" => Self::Else,
                b"enum" => Self::Enum,
                b"impl" => Self::Impl,
                b"loop" => Self::Loop,
                b"move" => Self::Move,
                b"priv" => Self::Priv,
                b"safe" => Self::Safe,
                b"self" => Self::SelfValue,
                b"Self" => Self::SelfType,
                b"true" => Self::True,
                b"type" => Self::Type,
                _ => Self::Other,
            },
            5 => match name {
                b"async" => Self::Async,
                b"await" => Self::Await,
                b"break" => Self::Break,
                b"const" => Self::Const,
                b"crate" => Self::Crate,
                b"false" => Self::False,
                b"final" => Self::Final,
                b"macro" => Self::Macro,
                b"match" => Self::Match,
                b"super" => Self::Super,
                b"trait" => Self::Trait,
                b"union" => Self::Union,
                b"where" => Self::Where,
                b"while" => Self::While,
                b"yield" => Self::Yield,
                _ => Self::Other,
            },
            6 => match name {
                b"become" => Self::Become,
                b"extern" => Self::Extern,
                b"return" => Self::Return,
                b"static" => Self::Static,
                b"struct" => Self::Struct,
                b"typeof" => Self::Typeof,
                b"unsafe" => Self::Unsafe,
                _ => Self::Other,
            },
            7 => match name {
                b"unsized" => Self::Unsized,
                b"virtual" => Self::Virtual,
                _ => Self::Other,
            },
            8 => match name {
                b"abstract" => Self::Abstract,
                b"continue" => Self::Continue,
                b"override" => Self::Override,
                _ => Self::Other,
            },
            11 if name == b"macro_rules" => Self::MacroRules,
            _ => Self::Other,
        }
    }

    /// Strict and reserved words cannot name ordinary identifiers.
    #[must_use]
    pub const fn reserved(self) -> bool {
        (self as u8) < (Self::Union as u8)
    }

    /// Path-segment keywords accepted where other reserved words are not.
    #[must_use]
    pub const fn path_keyword(self) -> bool {
        matches!(
            self,
            Self::SelfValue | Self::SelfType | Self::Super | Self::Crate
        )
    }
}

#[must_use]
pub fn identifier(kind: Kind, name: &str) -> bool {
    match kind {
        Kind::Identifier => !reserved(name),
        Kind::RawIdentifier => !matches!(name, "_" | "self" | "Self" | "super" | "crate"),
        _ => false,
    }
}

#[must_use]
pub fn reserved(name: &str) -> bool {
    Word::classify(name.as_bytes()).reserved()
}

#[cfg(test)]
mod tests {
    use super::Word;

    const RESERVED: [&str; 53] = [
        "_", "as", "async", "await", "break", "const", "continue", "crate", "dyn", "else", "enum",
        "extern", "false", "fn", "for", "if", "impl", "in", "let", "loop", "match", "mod", "move",
        "mut", "pub", "ref", "return", "self", "Self", "static", "struct", "super", "trait",
        "true", "type", "unsafe", "use", "where", "while", "abstract", "become", "box", "do",
        "final", "gen", "macro", "override", "priv", "try", "typeof", "unsized", "virtual",
        "yield",
    ];

    #[test]
    fn reserved_words_match_the_edition_list() {
        for name in RESERVED {
            assert!(super::reserved(name), "{name}");
        }
        let mut seen = std::collections::HashSet::new();
        for name in RESERVED {
            assert!(seen.insert(Word::classify(name.as_bytes())), "{name}");
        }
        for name in [
            "union",
            "macro_rules",
            "safe",
            "raw",
            "auto",
            "default",
            "r#fn",
            "",
            "Fn",
            "self_",
            "x",
            "selF",
        ] {
            assert!(!super::reserved(name), "{name}");
        }
        assert_eq!(Word::classify(b"union"), Word::Union);
        assert_eq!(Word::classify(b"macro_rules"), Word::MacroRules);
        assert_eq!(Word::classify(b"safe"), Word::Safe);
        assert_eq!(Word::classify(b"raw"), Word::Raw);
        assert_eq!(Word::classify(b"auto"), Word::Other);
    }
}
