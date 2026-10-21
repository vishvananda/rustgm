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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RmwOrder {
    Relaxed,
    Acquire,
    Release,
    AcquireRelease,
    Sequential,
}
impl RmwOrder {
    #[must_use]
    pub const fn encoding(self) -> u8 {
        match self {
            Self::Relaxed => 0,
            Self::Acquire => 2,
            Self::Release => 3,
            Self::AcquireRelease => 4,
            Self::Sequential => 5,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompareOrder {
    success: RmwOrder,
    failure: LoadOrder,
}
impl CompareOrder {
    #[must_use]
    pub const fn new(success: RmwOrder, failure: LoadOrder) -> Self {
        Self { success, failure }
    }
    #[must_use]
    pub const fn success_encoding(self) -> u8 {
        self.success.encoding()
    }
    #[must_use]
    pub const fn failure_encoding(self) -> u8 {
        self.failure.encoding()
    }
}
