//! Rust library ordering identities are distinct from backend memory-order codes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ordering {
    Relaxed,
    Release,
    Acquire,
    AcquireRelease,
    Sequential,
}
impl Ordering {
    pub const ALL: [Self; 5] = [
        Self::Relaxed,
        Self::Release,
        Self::Acquire,
        Self::AcquireRelease,
        Self::Sequential,
    ];
    #[must_use]
    pub const fn tag(self) -> u8 {
        match self {
            Self::Relaxed => 0,
            Self::Release => 1,
            Self::Acquire => 2,
            Self::AcquireRelease => 3,
            Self::Sequential => 4,
        }
    }
    #[must_use]
    pub fn from_tag(tag: u32) -> Option<Self> {
        Self::ALL.get(tag as usize).copied()
    }
    #[must_use]
    pub const fn load_valid(self) -> bool {
        !matches!(self, Self::Release | Self::AcquireRelease)
    }
    #[must_use]
    pub const fn store_valid(self) -> bool {
        !matches!(self, Self::Acquire | Self::AcquireRelease)
    }
}
