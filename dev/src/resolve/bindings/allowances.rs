//! Canonical feature symbols are data; actual consumers retain their authority.
use std::borrow::Cow;
use unicode_normalization::UnicodeNormalization;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KnownFeature {
    EditionPanic,
    PanicInternals,
    CoreIntrinsics,
    CfgUbChecks,
    CfgTargetHasAtomic,
}
impl KnownFeature {
    #[must_use]
    pub const fn name(self) -> FeatureName<'static> {
        FeatureName(Cow::Borrowed(match self {
            Self::EditionPanic => "edition_panic",
            Self::PanicInternals => "panic_internals",
            Self::CoreIntrinsics => "core_intrinsics",
            Self::CfgUbChecks => "cfg_ub_checks",
            Self::CfgTargetHasAtomic => "cfg_target_has_atomic",
        }))
    }
    pub(super) const fn mask(self) -> u8 {
        1 << self as u8
    }
}

/// A name already canonicalized by its declaration or the existing name arena.
#[derive(Clone)]
pub struct FeatureName<'a>(Cow<'a, str>);
impl<'a> FeatureName<'a> {
    /// Prepare one canonical identifier for repeated provenance queries.
    /// This does not enable a crate feature or authorize an API use.
    /// # Errors
    /// Rejects empty or invalid identifier spellings.
    pub fn new(spelling: &'a str) -> Result<Self, &'static str> {
        let name = spelling.strip_prefix("r#").unwrap_or(spelling);
        let mut characters = name.chars();
        if !characters
            .next()
            .is_some_and(|first| first == '_' || unicode_ident::is_xid_start(first))
            || name == "_"
            || !characters.all(unicode_ident::is_xid_continue)
        {
            return Err("invalid internal feature identifier");
        }
        Ok(Self(
            if name.is_ascii() || unicode_normalization::is_nfc(name) {
                Cow::Borrowed(name)
            } else {
                Cow::Owned(name.nfc().collect())
            },
        ))
    }
    #[must_use]
    pub fn spelling(&self) -> &str {
        &self.0
    }
}
