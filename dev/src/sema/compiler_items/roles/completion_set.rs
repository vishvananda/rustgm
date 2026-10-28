//! Completion membership stays in the existing cold ledger, indexed by real roles.
use crate::support::bindings::LangItem;

const BITS: usize = u64::BITS as usize;
pub(super) const WORDS: usize = LangItem::ALL.len().div_ceil(BITS);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::sema::compiler_items) struct RoleSet([u64; WORDS]);

impl RoleSet {
    pub(in crate::sema::compiler_items) const EMPTY: Self = Self([0; WORDS]);

    pub(in crate::sema::compiler_items) fn contains(&self, role: LangItem) -> bool {
        let index = role.index();
        self.0[index / BITS] & (1_u64 << (index % BITS)) != 0
    }

    pub(in crate::sema::compiler_items) fn insert(&mut self, role: LangItem) {
        let index = role.index();
        self.0[index / BITS] |= 1_u64 << (index % BITS);
    }

    pub(in crate::sema::compiler_items) fn is_empty(&self) -> bool {
        self.0.iter().all(|word| *word == 0)
    }

    pub(in crate::sema::compiler_items) fn extend(&mut self, other: Self) {
        for (own, incoming) in self.0.iter_mut().zip(other.0) {
            *own |= incoming;
        }
    }

    pub(in crate::sema::compiler_items) fn intersects(&self, other: Self) -> bool {
        self.0.iter().zip(other.0).any(|(a, b)| a & b != 0)
    }

    #[cfg(test)]
    pub(in crate::sema::compiler_items) fn remove(&mut self, role: LangItem) {
        let index = role.index();
        self.0[index / BITS] &= !(1_u64 << (index % BITS));
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/role_completions.rs"]
mod tests;
