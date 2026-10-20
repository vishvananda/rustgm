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

    #[must_use]
    pub const fn parent(self) -> Option<Self> {
        match self {
            Self::Fn => Some(Self::FnMut),
            Self::FnMut => Some(Self::FnOnce),
            Self::FnOnce => None,
        }
    }
}
