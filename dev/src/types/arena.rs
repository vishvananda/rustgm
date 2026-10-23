//! Lazy arena ownership survives moves and append-only growth, never replacement.
use super::Store;
use std::sync::Arc;

/// Kept private to the compiler. Store/Definitions themselves must not implement Clone.
#[derive(Clone)]
pub(crate) struct Identity(Arc<()>);
impl Identity {
    pub(super) fn new() -> Self {
        Self(Arc::new(()))
    }
    pub(super) fn same(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    /// Conservative control-block and alignment charge for one pinned Arc<()>.
    pub(crate) const fn allocation_bytes() -> usize {
        4 * size_of::<usize>()
    }
}
impl Store {
    pub(crate) fn arena_identity(&mut self) -> Identity {
        self.identity.get_or_insert_with(Identity::new).clone()
    }
    pub(crate) fn has_identity(&self, expected: &Identity) -> bool {
        self.identity
            .as_ref()
            .is_some_and(|found| found.same(expected))
    }
}
