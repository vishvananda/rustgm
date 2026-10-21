//! Language call traits, shared by resolution and typed callable contracts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CallTrait {
    Fn,
    FnMut,
    FnOnce,
}
impl CallTrait {
    pub const ALL: [Self; 3] = [Self::Fn, Self::FnMut, Self::FnOnce];

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Fn => "Fn",
            Self::FnMut => "FnMut",
            Self::FnOnce => "FnOnce",
        }
    }

    /// Canonical blanket reference implementations require this referent permission.
    pub(crate) const fn referent(self, mutable: bool) -> Option<Self> {
        if mutable {
            match self {
                Self::Fn => None,
                Self::FnMut | Self::FnOnce => Some(Self::FnMut),
            }
        } else {
            Some(Self::Fn)
        }
    }

    #[must_use]
    pub const fn parent(self) -> Option<Self> {
        match self {
            Self::Fn => Some(Self::FnMut),
            Self::FnMut => Some(Self::FnOnce),
            Self::FnOnce => None,
        }
    }
}
