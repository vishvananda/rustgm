//! Finite resource inputs are separate from source features and diagnostic policy.
/// Optional explicit upper caps on the input-proportional source-contract ledger.
/// An absent cap leaves only the ledger's own authenticated-input allowance.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SourceContractLimits {
    work_limit: Option<u32>,
    attempted_bytes: Option<u32>,
    retained_bytes: Option<u32>,
}
impl SourceContractLimits {
    /// Cap all three dimensions. Every value is a finite limit; zero rejects the
    /// first positive charge.
    #[must_use]
    pub const fn new(work_limit: u32, attempted_bytes: u32, retained_bytes: u32) -> Self {
        Self {
            work_limit: Some(work_limit),
            attempted_bytes: Some(attempted_bytes),
            retained_bytes: Some(retained_bytes),
        }
    }
    #[must_use]
    pub const fn with_work_limit(self, value: u32) -> Self {
        Self {
            work_limit: Some(value),
            ..self
        }
    }
    #[must_use]
    pub const fn with_attempted_bytes(self, value: u32) -> Self {
        Self {
            attempted_bytes: Some(value),
            ..self
        }
    }
    #[must_use]
    pub const fn with_retained_bytes(self, value: u32) -> Self {
        Self {
            retained_bytes: Some(value),
            ..self
        }
    }
    #[must_use]
    pub const fn work_limit(self) -> Option<u32> {
        self.work_limit
    }
    #[must_use]
    pub const fn attempted_bytes(self) -> Option<u32> {
        self.attempted_bytes
    }
    #[must_use]
    pub const fn retained_bytes(self) -> Option<u32> {
        self.retained_bytes
    }
}
