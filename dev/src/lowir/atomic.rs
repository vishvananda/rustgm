//! Operation-specific orders cannot encode illegal load/store memory orders.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadOrder {
    Relaxed,
    Acquire,
    Sequential,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreOrder {
    Relaxed,
    Release,
    Sequential,
}
impl LoadOrder {
    #[must_use]
    pub const fn encoding(self) -> u8 {
        match self {
            Self::Relaxed => 0,
            Self::Acquire => 2,
            Self::Sequential => 5,
        }
    }
}
impl StoreOrder {
    #[must_use]
    pub const fn encoding(self) -> u8 {
        match self {
            Self::Relaxed => 0,
            Self::Release => 3,
            Self::Sequential => 5,
        }
    }
}
