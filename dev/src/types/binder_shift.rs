//! Capture shifts depend on canonical structure and depth, never on caller permissions.
use super::{Store, TypeId, substitute::Bound};
use crate::support::hash::HashMap;
use std::collections::hash_map::Entry;

#[derive(Default)]
pub(super) struct Shifts {
    folders: HashMap<u16, Bound>,
    steps: usize,
}

impl Store {
    /// Share simultaneous argument insertion across independent parameter mappings.
    /// Each folder keeps nested binder depth in its own keys. Local slots stay local.
    /// The cumulative budget bounds both work and retained structural cache storage.
    pub(crate) fn shift_type(
        &mut self,
        ty: TypeId,
        amount: u16,
    ) -> Result<(TypeId, usize), &'static str> {
        let escaping = self.escaping_binders(ty).ok_or("invalid shifted type")?;
        if amount == 0 || escaping == 0 {
            return Ok((ty, 0));
        }
        let mut shifts = self.shifts.take().unwrap_or_default();
        let result = shifts.apply(self, ty, amount);
        // Failed folds retain their budget and any completed structural children.
        self.shifts = Some(shifts);
        result
    }
}

impl Shifts {
    fn apply(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        amount: u16,
    ) -> Result<(TypeId, usize), &'static str> {
        self.account(0)?;
        let mut work = 0;
        let folder = match self.folders.entry(amount) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                work += 1;
                entry.insert(Bound::shift(amount))
            }
        };
        let before = folder.steps;
        let result = folder.apply(store, ty);
        work += folder.steps.saturating_sub(before);
        self.account(work)?;
        result.map(|ty| (ty, work))
    }

    fn account(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps > 1_000_000 {
            return Err("capture shift work budget exceeded");
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/capture_shifts.rs"]
mod tests;
