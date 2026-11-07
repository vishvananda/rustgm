//! The ledger's allowance grows only with authenticated admitted source bytes.
//!
//! Factors are sized from the original-core payer attribution recorded in
//! `docs/parallel/macro-budget.md`: genuinely linear payers used 0.57 work units,
//! 16.7 attempted bytes and a 1.88-byte retained peak per admitted byte.
use crate::expand::config::SourceContractLimits;

/// Work units granted per original byte of an admitted source file.
const WORK_PER_BYTE: usize = 2;
/// Attempted-allocation bytes granted per original byte.
const ATTEMPTED_PER_BYTE: usize = 64;
/// Retained-payload bytes granted per original byte.
const RETAINED_PER_BYTE: usize = 8;

/// Granted allowance before explicit caps; it never shrinks or refunds work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Allowance {
    pub(super) work: usize,
    pub(super) attempted: usize,
    pub(super) retained: usize,
}
impl Allowance {
    /// Fixed per-crate setup floor: provider tables, prelude and builtin
    /// vocabularies are charged before most source is read. It equals the
    /// former whole-crate default, so no previously accepted input loses budget.
    pub(super) const SETUP: Self = Self {
        work: 1_000_000,
        attempted: 64 * 1024 * 1024,
        retained: 64 * 1024 * 1024,
    };
    /// Test ledgers isolate explicit-cap mechanics from source admission.
    #[cfg(test)]
    pub(super) const UNBOUNDED: Self = Self {
        work: usize::MAX,
        attempted: usize::MAX,
        retained: usize::MAX,
    };
    /// One admitted source grants proportionally to its original bytes.
    /// Expansion output, copied instances and repeated requests are not admissions.
    pub(super) const fn admit(self, bytes: usize) -> Self {
        Self {
            work: self
                .work
                .saturating_add(bytes.saturating_mul(WORK_PER_BYTE)),
            attempted: self
                .attempted
                .saturating_add(bytes.saturating_mul(ATTEMPTED_PER_BYTE)),
            retained: self
                .retained
                .saturating_add(bytes.saturating_mul(RETAINED_PER_BYTE)),
        }
    }
    /// Each explicit cap bounds its dimension; absent caps leave the grant.
    pub(super) fn capped(self, limits: SourceContractLimits) -> Self {
        let cap =
            |granted: usize, cap: Option<u32>| cap.map_or(granted, |cap| granted.min(cap as usize));
        Self {
            work: cap(self.work, limits.work_limit()),
            attempted: cap(self.attempted, limits.attempted_bytes()),
            retained: cap(self.retained, limits.retained_bytes()),
        }
    }
}
impl Default for Allowance {
    /// A context-free scratch ledger reads no admitted source.
    fn default() -> Self {
        Self::SETUP
    }
}
