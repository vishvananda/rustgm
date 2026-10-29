//! Shared identities for builtin marker names and typed obligations.
/// Stable auto traits in the course's language profile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AutoTrait {
    Send,
    Sync,
    Unpin,
    UnwindSafe,
    RefUnwindSafe,
    /// Internal structural rule; actual source Freeze retains its Trait application.
    Freeze,
    /// Source-only structural rule; actual `UnsafeUnpin` remains a Trait application.
    UnsafeUnpin,
}
impl AutoTrait {
    pub const ALL: [Self; 5] = [
        Self::Send,
        Self::Sync,
        Self::Unpin,
        Self::UnwindSafe,
        Self::RefUnwindSafe,
    ];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Send => "Send",
            Self::Sync => "Sync",
            Self::Unpin => "Unpin",
            Self::UnwindSafe => "UnwindSafe",
            Self::RefUnwindSafe => "RefUnwindSafe",
            Self::Freeze => "Freeze",
            Self::UnsafeUnpin => "UnsafeUnpin",
        }
    }
}
