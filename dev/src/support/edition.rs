//! Defining language semantics, shared by source parsing and imported bodies.
//! This identity is not a metadata wire tag or a claim of driver support.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Edition {
    Rust2015,
    Rust2018,
    Rust2021,
    #[default]
    Rust2024,
}

/// An edition before 2021, whose differing rules are each selected or fenced.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Legacy {
    Rust2015,
    Rust2018,
}

impl Edition {
    /// Returns the older edition for rules whose pre-2021 behavior differs.
    #[must_use]
    pub const fn legacy(self) -> Option<Legacy> {
        match self {
            Self::Rust2015 => Some(Legacy::Rust2015),
            Self::Rust2018 => Some(Legacy::Rust2018),
            Self::Rust2021 | Self::Rust2024 => None,
        }
    }

    /// Rules that changed only in 2024 keep one shared behavior before it.
    #[must_use]
    pub const fn before_2024(self) -> bool {
        match self {
            Self::Rust2015 | Self::Rust2018 | Self::Rust2021 => true,
            Self::Rust2024 => false,
        }
    }
}

/// A pre-2021 rule `RustGM` rejects explicitly instead of applying a newer rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum LegacyRule {
    /// `async`, `await`, `dyn` and `try` are identifiers in Rust 2015.
    LegacyKeywords,
    /// Rust 2015 trait methods may omit parameter patterns.
    AnonymousParameters,
    /// `ident#`, `ident"` and `ident'` are separate tokens before 2021.
    UnreservedPrefixes,
    /// A path type naming a trait is a trait object before 2021.
    BareTraitObjects,
    /// `a...b` is an inclusive range pattern before 2021.
    DotDotDotPatterns,
    /// Older panic macros accept a non-format literal or payload.
    PanicMacros,
    /// Older array method probes skip by-value `IntoIterator`.
    ArrayIntoIterator,
}

impl LegacyRule {
    /// The typed diagnostic for an unimplemented legacy rule.
    #[must_use]
    pub const fn unsupported(self, edition: Legacy) -> &'static str {
        macro_rules! messages {
            ($($rule:ident => $text:literal,)*) => {
                match (self, edition) {
                    $(
                        (Self::$rule, Legacy::Rust2015) => {
                            concat!($text, " is not implemented for edition 2015")
                        }
                        (Self::$rule, Legacy::Rust2018) => {
                            concat!($text, " is not implemented for edition 2018")
                        }
                    )*
                }
            };
        }
        messages! {
            LegacyKeywords => "async, await, dyn or try as a Rust 2015 identifier",
            AnonymousParameters => "anonymous trait method parameters",
            UnreservedPrefixes => "unreserved identifier prefixes",
            BareTraitObjects => "trait objects without dyn",
            DotDotDotPatterns => "`...` range patterns",
            PanicMacros => "the pre-2021 panic macro form",
            ArrayIntoIterator => "array method dispatch that skips by-value into_iter",
        }
    }
}
