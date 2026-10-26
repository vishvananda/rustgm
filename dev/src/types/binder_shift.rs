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
    /// Cumulative bounded relocation work, including failed attempts. Callers
    /// may attribute a delta without changing the relocation budget owner.
    pub(crate) fn capture_shift_steps(&self) -> usize {
        self.shifts.as_deref().map_or(0, |shifts| shifts.steps)
    }
    /// Read a completed capture relocation in this Store without allocating or
    /// interning. A miss does not mean identity, and failed budgets stay failed.
    pub(crate) fn prepared_shift(
        &self,
        ty: TypeId,
        amount: u16,
    ) -> Result<Option<TypeId>, &'static str> {
        let escaping = self.escaping_binders(ty).ok_or("invalid shifted type")?;
        let folder = self
            .shifts
            .as_deref()
            .and_then(|shifts| shifts.folders.get(&amount));
        if self
            .shifts
            .as_deref()
            .is_some_and(|shifts| shifts.steps > 1_000_000)
        {
            return Err("capture shift work budget exceeded");
        }
        if folder.is_some_and(|folder| folder.steps > 1_000_000) {
            return Err("lifetime substitution work budget exceeded");
        }
        if amount == 0 || escaping == 0 {
            return Ok(Some(ty));
        }
        folder.map_or(Ok(None), |folder| folder.completed_type(self, ty))
    }
    /// Share simultaneous argument insertion across independent parameter mappings.
    /// Each folder keeps nested binder depth in its own keys. Local slots stay local.
    /// The cumulative budget bounds both work and retained structural cache storage.
    pub(crate) fn shift_type(
        &mut self,
        ty: TypeId,
        amount: u16,
    ) -> Result<(TypeId, usize), &'static str> {
        self.shift_type_observed::<false>(ty, amount, &|_, _| Ok(()))
    }
    pub(crate) fn shift_type_charged(
        &mut self,
        ty: TypeId,
        amount: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(TypeId, usize), &'static str> {
        self.shift_type_observed::<true>(ty, amount, charge)
    }
    fn shift_type_observed<const CHARGED: bool>(
        &mut self,
        ty: TypeId,
        amount: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(TypeId, usize), &'static str> {
        if CHARGED {
            charge(1, 0)?;
        }
        let escaping = self.escaping_binders(ty).ok_or("invalid shifted type")?;
        if amount == 0 || escaping == 0 {
            return Ok((ty, 0));
        }
        if CHARGED && self.shifts.is_none() {
            charge(1, size_of::<Shifts>())?;
        }
        let mut shifts = self.shifts.take().unwrap_or_default();
        let result = shifts.apply_observed::<CHARGED>(self, ty, amount, charge);
        // Failed folds retain their budget and any completed structural children.
        self.shifts = Some(shifts);
        result
    }
}

impl Shifts {
    fn apply_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        amount: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(TypeId, usize), &'static str> {
        self.account(0)?;
        if CHARGED {
            let growth = crate::support::retained::Growth::map::<u16, Bound>(
                self.folders.len(),
                self.folders.capacity(),
                usize::from(!self.folders.contains_key(&amount)),
            );
            charge(growth.work, growth.bytes)?;
        }
        let mut work = 0;
        let folder = match self.folders.entry(amount) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                work += 1;
                entry.insert(Bound::shift(amount))
            }
        };
        let before = folder.steps;
        let result = folder.apply_inside_observed::<CHARGED>(store, ty, 0, charge);
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
