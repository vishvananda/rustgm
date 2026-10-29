//! Finite resource inputs are separate from source features and diagnostic policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SourceContractLimits {
    work_limit: u32,
    attempted_bytes: u32,
    retained_bytes: u32,
}
impl Default for SourceContractLimits {
    fn default() -> Self {
        Self::new(1_000_000, 64 * 1024 * 1024, 64 * 1024 * 1024)
    }
}
impl SourceContractLimits {
    /// Every value is a finite limit; zero rejects the first positive charge.
    #[must_use]
    pub const fn new(work_limit: u32, attempted_bytes: u32, retained_bytes: u32) -> Self {
        Self {
            work_limit,
            attempted_bytes,
            retained_bytes,
        }
    }
    #[must_use]
    pub const fn work_limit(self) -> u32 {
        self.work_limit
    }
    #[must_use]
    pub const fn attempted_bytes(self) -> u32 {
        self.attempted_bytes
    }
    #[must_use]
    pub const fn retained_bytes(self) -> u32 {
        self.retained_bytes
    }
}
