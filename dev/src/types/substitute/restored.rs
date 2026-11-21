//! A fixed restore map owns a persistent budget, never an implicit retained interval.
use super::{
    Action, Bound, Store, TypeId,
    bounded::{self, Budget},
    region_mapping::{Mapping, Mode, Retained},
};
use crate::support::hash::HashMap;
use crate::types::RegionId;
use std::{ops::Range, sync::Arc};

pub(super) struct Owner {
    pub(super) budget: Budget,
    pub(super) external: usize,
    pub(super) transport: Option<Box<Transport>>,
}

/// An exact owned interval and the first region in its translated interval.
pub(crate) struct RegionRun {
    pub source: Range<u32>,
    pub target: u32,
}
impl RegionRun {
    pub(crate) fn at_or_after<'a>(
        runs: &'a [Self],
        region: u32,
        steps: &mut usize,
    ) -> Option<&'a Self> {
        let index = runs.partition_point(|run| {
            *steps = steps.saturating_add(1);
            run.source.end <= region
        });
        runs.get(index)
    }
}

pub(super) struct Transport {
    runs: Arc<[RegionRun]>,
    universals: u32,
}
impl Transport {
    pub(super) fn bytes(&self) -> usize {
        size_of::<Self>() + size_of_val(self.runs.as_ref())
    }

    pub(super) fn region(&self, id: RegionId, steps: &mut usize) -> Result<RegionId, &'static str> {
        if let Some((depth, slot)) = id.bound_parts() {
            return if depth == 0 && u32::from(slot) < self.universals {
                Ok(id)
            } else {
                Err(super::UNMAPPED_LIFETIME)
            };
        }
        let run = RegionRun::at_or_after(&self.runs, id.0, steps)
            .filter(|run| run.source.contains(&id.0))
            .ok_or(super::UNMAPPED_LIFETIME)?;
        Ok(RegionId(run.target + (id.0 - run.source.start)))
    }
}
impl Bound {
    pub(crate) const fn restore_transport_overhead() -> usize {
        size_of::<Mapping>() + size_of::<Owner>() + size_of::<Transport>()
    }

    pub(crate) fn restore_budgeted_regions(
        replacements: HashMap<RegionId, RegionId>,
        work: usize,
        bytes: usize,
    ) -> Result<Self, &'static str> {
        Self::with_restore_owner(
            replacements,
            Owner {
                budget: Budget::new(bytes, work),
                external: 0,
                transport: None,
            },
        )
    }

    /// Reuse the caller's ordered, disjoint translation ranges in the fold.
    pub(crate) fn restore_translated_regions(
        replacements: HashMap<RegionId, RegionId>,
        runs: Arc<[RegionRun]>,
        universals: u32,
        work: usize,
        bytes: usize,
    ) -> Result<Self, &'static str> {
        Self::with_restore_owner(
            replacements,
            Owner {
                budget: Budget::new(bytes, work),
                external: 0,
                transport: Some(Box::new(Transport { runs, universals })),
            },
        )
    }

    fn with_restore_owner(
        replacements: HashMap<RegionId, RegionId>,
        owner: Owner,
    ) -> Result<Self, &'static str> {
        let mut fold = Self::restore_regions(replacements);
        let Action::Regions(mapping) = &mut fold.action else {
            return Err("missing restore map");
        };
        mapping.mode = Mode::Restore {
            retained: Retained::Budget(Box::new(owner)),
        };
        fold.region_budget()?
            .ok_or("missing restore budget")?
            .check_bytes(fold.region_retained_bytes()?)?;
        Ok(fold)
    }
    pub(super) fn region_budget(&self) -> Result<Option<Budget>, &'static str> {
        let Some(owner) = self.region_owner() else {
            return Ok(None);
        };
        if self.parameters.is_some() || !self.arguments.is_empty() {
            return Err("bounded region map cannot bind parameters or arguments");
        }
        Ok(Some(owner.budget))
    }
    fn region_owner(&self) -> Option<&Owner> {
        if let Action::Regions(mapping) = &self.action
            && let Mode::Restore {
                retained: Retained::Budget(owner),
            }
            | Mode::ObserveBudget(owner) = &mapping.mode
        {
            Some(owner)
        } else {
            None
        }
    }
    fn region_owner_mut(&mut self) -> Option<&mut Owner> {
        if let Action::Regions(mapping) = &mut self.action
            && let Mode::Restore {
                retained: Retained::Budget(owner),
            }
            | Mode::ObserveBudget(owner) = &mut mapping.mode
        {
            Some(owner)
        } else {
            None
        }
    }
    pub(super) fn region_work(&mut self, count: usize) -> Result<(), &'static str> {
        if let Some(budget) = self.region_budget()? {
            self.steps = self.steps.saturating_add(count);
            budget.check_steps(self.steps)?;
        }
        Ok(())
    }
    pub(super) fn region_retained_bytes(&self) -> Result<usize, &'static str> {
        use crate::support::retained::{map, sum, vector};
        let owner = self
            .region_owner()
            .ok_or("folder has no fixed restore budget")?;
        self.region_budget()?;
        let Action::Regions(mapping) = &self.action else {
            return Err("missing fixed restore map");
        };
        Ok(sum([
            size_of::<Self>(),
            size_of::<Mapping>(),
            size_of::<Owner>(),
            owner.transport.as_deref().map_or(0, Transport::bytes),
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
    pub(super) fn apply_budgeted_regions_inside(
        &mut self,
        store: &mut Store,
        ty: TypeId,
        depth: u16,
        budget: Budget,
    ) -> Result<TypeId, &'static str> {
        self.region_work(1)?;
        let retained = self.region_retained_bytes()?;
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
    pub(super) fn reserve_region_scratch(&mut self, bytes: usize) -> Result<usize, &'static str> {
        let Some(budget) = self.region_budget()? else {
            return Ok(0);
        };
        self.region_work(1)?;
        budget.check_bytes(self.region_retained_bytes()?.saturating_add(bytes))?;
        let owner = self
            .region_owner_mut()
            .ok_or("restore budget owner changed")?;
        owner.external = owner
            .external
            .checked_add(bytes)
            .ok_or("restore scratch overflow")?;
        Ok(bytes)
    }
    pub(super) fn release_region_scratch(&mut self, bytes: usize) -> Result<(), &'static str> {
        if bytes != 0 {
            let owner = self
                .region_owner_mut()
                .ok_or("restore budget owner changed")?;
            owner.external = owner
                .external
                .checked_sub(bytes)
                .ok_or("restore scratch was not reserved")?;
        }
        Ok(())
    }
    pub(crate) fn validate_restore_owner(&self) -> Result<(), &'static str> {
        if !matches!(&self.action, Action::Regions(mapping) if matches!(mapping.mode,
            Mode::Restore { retained: Retained::Budget(_) }))
        {
            return Err("folder lost its fixed restore owner");
        }
        self.validate_region_budget()
    }
    pub(super) fn validate_region_budget(&self) -> Result<(), &'static str> {
        let owner = self
            .region_owner()
            .ok_or("folder lost its fixed restore owner")?;
        if owner.external != 0 {
            return Err("restore scratch escaped its operation");
        }
        let budget = self
            .region_budget()?
            .ok_or("folder lost its fixed restore owner")?;
        budget.check_steps(self.steps)?;
        budget.check_bytes(self.region_retained_bytes()?)
    }
    pub(crate) fn tighten_restore_bytes(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.validate_restore_owner()?;
        let owner = self
            .region_owner_mut()
            .ok_or("folder lost its fixed restore owner")?;
        owner.budget = owner.budget.tightened(bytes);
        self.region_budget()?
            .ok_or("missing restore budget")?
            .check_bytes(self.region_retained_bytes()?)
    }

    /// A materialized body owns its canonical types. Keep the namespace and
    /// cumulative work while releasing the memo and buffers used to build it.
    pub(crate) fn release_restore_workspace(&mut self) {
        self.completed = HashMap::default();
        self.applications = None;
        self.work = Vec::new();
        self.children = Vec::new();
        self.regions = Vec::new();
        self.generic = Vec::new();
    }
}
#[cfg(test)]
#[path = "../../../tests/invariants/budgeted_restores.rs"]
mod tests;
