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
    MutableSlice,
    MutableArray,
    SliceMutableIterator,
    SliceMutableMethod,
}
impl Collection {
    pub const SHARED: [Self; 4] = [
        Self::SharedSlice,
        Self::SharedArray,
        Self::SliceIterator,
        Self::SliceMethod,
    ];
    pub const MUTABLE: [Self; 4] = [
        Self::MutableSlice,
        Self::MutableArray,
        Self::SliceMutableIterator,
        Self::SliceMutableMethod,
    ];
    #[must_use]
    pub const fn mutable(self) -> bool {
        (self as usize) >= 4
    }
    #[must_use]
    pub const fn index(self) -> usize {
        self as usize % 4
    }
    #[must_use]
    pub const fn next(self) -> bool {
        matches!(self, Self::SliceIterator | Self::SliceMutableIterator)
    }
    #[must_use]
    pub const fn method(self) -> bool {
        matches!(self, Self::SliceMethod | Self::SliceMutableMethod)
    }
    #[must_use]
    pub const fn array(self) -> bool {
        matches!(self, Self::SharedArray | Self::MutableArray)
    }
    #[must_use]
    pub const fn trait_(self) -> Option<Trait> {
        if self.method() {
            None
        } else if self.next() {
            Some(Trait::Iterator)
        } else {
            Some(Trait::IntoIterator)
        }
    }
}
