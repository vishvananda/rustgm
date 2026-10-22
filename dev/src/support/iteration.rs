//! Canonical iteration roles carry no type, selection or execution permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Trait {
    Iterator,
    IntoIterator,
}
impl Trait {
    pub const ALL: [Self; 2] = [Self::Iterator, Self::IntoIterator];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Iterator => "Iterator",
            Self::IntoIterator => "IntoIterator",
        }
    }
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::Iterator => Method::Next,
            Self::IntoIterator => Method::IntoIter,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Method {
    Next,
    IntoIter,
}
impl Method {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Next => "next",
            Self::IntoIter => "into_iter",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Blanket {
    Identity,
    MutableReference,
}

/// Real canonical collection implementation/method owners, separate from blankets.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Collection {
    SharedSlice,
    SharedArray,
    SliceIterator,
    SliceMethod,
}
impl Collection {
    pub const ALL: [Self; 4] = [
        Self::SharedSlice,
        Self::SharedArray,
        Self::SliceIterator,
        Self::SliceMethod,
    ];
    #[must_use]
    pub const fn trait_(self) -> Option<Trait> {
        match self {
            Self::SharedSlice | Self::SharedArray => Some(Trait::IntoIterator),
            Self::SliceIterator => Some(Trait::Iterator),
            Self::SliceMethod => None,
        }
    }
}
