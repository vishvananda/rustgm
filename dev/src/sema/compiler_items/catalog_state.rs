//! The catalog state that a trait answer may read and that is written in
//! place after the catalog is installed. Bindings and the provider identity
//! are fixed at installation; every other in-place write is invisible to
//! trait selection or is read only through a lookup that refuses or defers
//! until it completes.
use super::{Catalog, roles::RoleSet};
use crate::resolve::model::DefId;

/// Two equal states give every closed trait goal the same answer, so a table
/// of answers compares this state by content instead of the catalog's address.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::sema) struct SelectionState {
    roles: Option<RoleSet>,
    methods: [Option<DefId>; 3],
    /// Pointee, discriminant, range and field contracts, in that order.
    evidence: [bool; 4],
    sizing: Option<[DefId; 3]>,
    destruction: Option<DefId>,
    async_destruction: Option<DefId>,
}

impl Catalog {
    pub(in crate::sema) fn selection_state(&self) -> SelectionState {
        let ledger = self.traits.as_deref();
        SelectionState {
            roles: ledger.map(|ledger| ledger.roles),
            methods: ledger.map_or([None; 3], |ledger| ledger.methods),
            evidence: ledger.map_or([false; 4], |ledger| {
                [
                    ledger.pointee.is_some(),
                    ledger.discriminant.is_some(),
                    ledger.range.is_some(),
                    ledger.field.is_some(),
                ]
            }),
            sizing: self.sizing,
            destruction: self.destruction_registration,
            async_destruction: self.async_destruction_registration,
        }
    }
}

#[cfg(test)]
impl SelectionState {
    /// The same catalog after its destructor registration completed in place.
    pub(in crate::sema) const fn with_destruction(mut self, definition: DefId) -> Self {
        self.destruction = Some(definition);
        self
    }
}
