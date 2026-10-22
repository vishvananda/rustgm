//! A fixed restore map owns a persistent budget, never an implicit retained interval.
use super::{
    Action, Bound, Store, TypeId,
    bounded::{self, Budget},
    region_mapping::{Mapping, Mode, Retained},
};
use crate::types::RegionId;
use std::collections::HashMap;
pub(super) struct Owner {
    budget: Budget,
    external: usize,
}
impl Bound {
    pub(crate) const fn restore_owner_overhead() -> usize {
        size_of::<Mapping>() + size_of::<Owner>()
    }

    pub(crate) fn restore_budgeted_regions(
        replacements: HashMap<RegionId, RegionId>,
        work: usize,
        bytes: usize,
    ) -> Result<Self, &'static str> {
        let mut fold = Self::restore_regions(replacements);
        let Action::Regions(mapping) = &mut fold.action else {
            return Err("missing restore map");
        };
        mapping.mode = Mode::Restore {
            retained: Retained::Budget(Box::new(Owner {
                budget: Budget::new(bytes, work),
                external: 0,
            })),
        };
        fold.restore_budget()?
            .ok_or("missing restore budget")?
            .check_bytes(fold.restore_retained_bytes()?)?;
        Ok(fold)
    }
    pub(super) fn restore_budget(&self) -> Result<Option<Budget>, &'static str> {
        let Some(owner) = self.restore_owner() else {
            return Ok(None);
        };
        if self.parameters.is_some() || !self.arguments.is_empty() {
            return Err("bounded restore map cannot bind parameters or arguments");
        }
        Ok(Some(owner.budget))
    }
    fn restore_owner(&self) -> Option<&Owner> {
        if let Action::Regions(mapping) = &self.action
            && let Mode::Restore {
                retained: Retained::Budget(owner),
            } = &mapping.mode
        {
            Some(owner)
        } else {
            None
        }
    }
    fn restore_owner_mut(&mut self) -> Option<&mut Owner> {
        if let Action::Regions(mapping) = &mut self.action
            && let Mode::Restore {
                retained: Retained::Budget(owner),
            } = &mut mapping.mode
        {
            Some(owner)
        } else {
            None
        }
    }
    pub(super) fn restore_work(&mut self, count: usize) -> Result<(), &'static str> {
        if let Some(budget) = self.restore_budget()? {
            self.steps = self.steps.saturating_add(count);
            budget.check_steps(self.steps)?;
        }
        Ok(())
    }
    pub(super) fn restore_retained_bytes(&self) -> Result<usize, &'static str> {
        use crate::support::retained::{map, sum, vector};
        let owner = self
            .restore_owner()
            .ok_or("folder has no fixed restore budget")?;
        self.restore_budget()?;
        let Action::Regions(mapping) = &self.action else {
            return Err("missing fixed restore map");
        };
        Ok(sum([
            size_of::<Self>(),
            size_of::<Mapping>(),
            size_of::<Owner>(),
            map(&mapping.replacements),
            map(&self.completed).saturating_mul(4),
            vector(&self.arguments),
            vector(&self.work),
            vector(&self.children),
            vector(&self.regions),
            vector(&self.generic),
            owner.external,
            self.applications.as_ref().map_or(0, |a| {
                size_of_val(a.as_ref()).saturating_add(a.allocation().bytes.saturating_mul(4))
            }),
        ]))
    }
    pub(super) fn apply_restored_inside(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        depth: u16,
        budget: Budget,
    ) -> Result<TypeId, &'static str> {
        self.restore_work(1)?;
        let retained = self.restore_retained_bytes()?;
        budget.check_bytes(retained)?;
        let key = (ty, self.cache_depth(depth));
        if let Some(value) = self.result(store, key)? {
            return Ok(value);
        }
        budget.check_bytes(
            retained.saturating_add(bounded::growth::<(super::Key, bool)>(
                self.work.len().saturating_add(1),
            )),
        )?;
        self.work
            .try_reserve(1)
            .map_err(|_| "bounded restore pending allocation failed")?;
        self.work.push((key, false));
        let result = self
            .drain::<true>(store, &budget)
            .and_then(|()| self.result(store, key)?.ok_or("missing restored root"));
        self.work.clear();
        result
    }
    /// Stack-local vectors/cache insertion reservations remain visible during nested folds.
    pub(super) fn reserve_restore_scratch(&mut self, bytes: usize) -> Result<usize, &'static str> {
        let Some(budget) = self.restore_budget()? else {
            return Ok(0);
        };
        self.restore_work(1)?;
        budget.check_bytes(self.restore_retained_bytes()?.saturating_add(bytes))?;
        let owner = self
            .restore_owner_mut()
            .ok_or("restore budget owner changed")?;
        owner.external = owner
            .external
            .checked_add(bytes)
            .ok_or("restore scratch overflow")?;
        Ok(bytes)
    }
    pub(super) fn release_restore_scratch(&mut self, bytes: usize) -> Result<(), &'static str> {
        if bytes != 0 {
            let owner = self
                .restore_owner_mut()
                .ok_or("restore budget owner changed")?;
            owner.external = owner
                .external
                .checked_sub(bytes)
                .ok_or("restore scratch was not reserved")?;
        }
        Ok(())
    }
    pub(crate) fn validate_restore_owner(&self) -> Result<(), &'static str> {
        let owner = self
            .restore_owner()
            .ok_or("folder lost its fixed restore owner")?;
        if owner.external != 0 {
            return Err("restore scratch escaped its operation");
        }
        let budget = self
            .restore_budget()?
            .ok_or("folder lost its fixed restore owner")?;
        budget.check_steps(self.steps)?;
        budget.check_bytes(self.restore_retained_bytes()?)
    }
    pub(crate) fn tighten_restore_bytes(&mut self, bytes: usize) -> Result<(), &'static str> {
        let owner = self
            .restore_owner_mut()
            .ok_or("folder lost its fixed restore owner")?;
        owner.budget = owner.budget.tightened(bytes);
        self.restore_budget()?
            .ok_or("missing restore budget")?
            .check_bytes(self.restore_retained_bytes()?)
    }
}
#[cfg(test)]
#[path = "../../../tests/invariants/budgeted_restores.rs"]
mod tests;
