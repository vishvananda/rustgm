//! Capture shifts depend on canonical structure and depth, never on caller permissions.
use super::{Store, TypeId, substitute::Bound};
use crate::support::{hash::HashMap, retained};
use std::collections::hash_map::Entry;

type Key = (TypeId, u16);

#[derive(Default)]
pub(super) struct Shifts {
    folders: HashMap<u16, Bound>,
    prepared: HashMap<Key, TypeId>,
    cache_bytes: usize,
    cache_full: bool,
    steps: usize,
}

impl Store {
    /// Cumulative relocation work, including failed attempts. Each fold has
    /// its own work limit.
    pub(crate) fn capture_shift_steps(&self) -> usize {
        self.shifts.as_deref().map_or(0, |shifts| shifts.steps)
    }

    /// Seal the exact relocation for later read-only intrinsic validation.
    pub(crate) fn prepare_shift(&mut self, ty: TypeId, amount: u16) -> Result<(), &'static str> {
        if self.prepared_shift(ty, amount)?.is_some() {
            return Ok(());
        }
        let (shifted, _) = self.shift_type(ty, amount)?;
        let shifts = self
            .shifts
            .as_mut()
            .ok_or("missing prepared capture shift")?;
        if retained::new_map::<Key, TypeId>(shifts.prepared.len().saturating_add(1)) > 2 << 20 {
            return Err("prepared capture shifts exceed their storage budget");
        }
        shifts.prepared.insert((ty, amount), shifted);
        Ok(())
    }

    /// Read the explicitly prepared result without recomputing or interning.
    pub(crate) fn prepared_shift(
        &self,
        ty: TypeId,
        amount: u16,
    ) -> Result<Option<TypeId>, &'static str> {
        let escaping = self.escaping_binders(ty).ok_or("invalid shifted type")?;
        if amount == 0 || escaping == 0 {
            return Ok(Some(ty));
        }
        Ok(self
            .shifts
            .as_deref()
            .and_then(|shifts| shifts.prepared.get(&(ty, amount)).copied()))
    }

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
        self.shifts = Some(shifts);
        result
    }
}

impl Shifts {
    const CACHE_LIMIT: usize = 32 << 20;

    fn apply_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        amount: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(TypeId, usize), &'static str> {
        if let Some(folder) = self.folders.get(&amount)
            && let Some(shifted) = folder.completed_type(store, ty)?
        {
            return Ok((shifted, 0));
        }
        if self.cache_full {
            let mut folder = Bound::shift(amount);
            let result = folder.apply_inside_observed::<CHARGED>(store, ty, 0, charge);
            let work = folder.steps.saturating_add(1);
            self.steps = self.steps.saturating_add(work);
            return result.map(|shifted| (shifted, work));
        }

        if CHARGED {
            let growth = retained::Growth::map::<u16, Bound>(
                self.folders.len(),
                self.folders.capacity(),
                usize::from(!self.folders.contains_key(&amount)),
            );
            charge(growth.work, growth.bytes)?;
        }
        let (folder, startup) = match self.folders.entry(amount) {
            Entry::Occupied(entry) => (entry.into_mut(), 0_usize),
            Entry::Vacant(entry) => (entry.insert(Bound::shift(amount)), 1),
        };
        let before = folder.retained_allocation().0;
        let result = folder.apply_inside_observed::<CHARGED>(store, ty, 0, charge);
        let work = std::mem::take(&mut folder.steps).saturating_add(startup);
        let after = folder.retained_allocation().0;
        self.steps = self.steps.saturating_add(work);
        self.cache_bytes = self
            .cache_bytes
            .saturating_sub(before)
            .saturating_add(after);
        if self
            .cache_bytes
            .saturating_add(retained::map(&self.folders))
            > Self::CACHE_LIMIT
        {
            self.folders.remove(&amount);
            self.cache_bytes = self.cache_bytes.saturating_sub(after);
            self.cache_full = true;
        }
        result.map(|shifted| (shifted, work))
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/capture_shifts.rs"]
mod tests;
