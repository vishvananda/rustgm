//! Bounded observation records identities without granting restoration or proof authority.
use super::{
    Action, Bound, RegionId,
    bounded::{self, Budget},
    region_mapping::{Mapping, Mode},
    restored::Owner,
};
use crate::support::hash::HashMap;

impl Bound {
    pub(crate) fn observe_budgeted_regions(
        work: usize,
        bytes: usize,
    ) -> Result<Self, &'static str> {
        let overhead = size_of::<Self>() + size_of::<Mapping>() + size_of::<Owner>();
        Budget::new(bytes, work).check_bytes(overhead)?;
        let mut fold = Self::new(Vec::new());
        fold.action = Action::Regions(Box::new(Mapping {
            mode: Mode::ObserveBudget(Box::new(Owner {
                budget: Budget::new(bytes, work),
                external: 0,
            })),
            replacements: HashMap::default(),
        }));
        Ok(fold)
    }

    pub(super) fn prepare_observation(
        &mut self,
        region: RegionId,
        depth: u16,
    ) -> Result<(), &'static str> {
        let Action::Regions(mapping) = &self.action else {
            return Ok(());
        };
        if !matches!(mapping.mode, Mode::ObserveBudget(_)) || region == RegionId::STATIC {
            return Ok(());
        }
        let key = match region.bound_parts() {
            Some((binder, _)) if binder < depth => return Ok(()),
            Some((binder, slot)) => RegionId::bound(binder - depth, slot)?,
            None => region,
        };
        if mapping.replacements.contains_key(&key) {
            return Ok(());
        }
        let planned = crate::support::retained::new_map::<RegionId, RegionId>(
            mapping.replacements.len().saturating_add(1),
        );
        self.region_budget()?
            .ok_or("missing observation budget")?
            .check_bytes(self.region_retained_bytes()?.saturating_add(planned))?;
        let Action::Regions(mapping) = &mut self.action else {
            return Err("observation owner changed");
        };
        mapping
            .replacements
            .try_reserve(1)
            .map_err(|_| "bounded observation map allocation failed")
    }

    pub(crate) fn validate_observation_owner(&self) -> Result<(), &'static str> {
        if !matches!(&self.action, Action::Regions(mapping)
            if matches!(mapping.mode, Mode::ObserveBudget(_)))
        {
            return Err("folder lost its observation owner");
        }
        self.validate_region_budget()
    }

    /// Account the output while the observer's completed cache is still live.
    /// The caller separately accounts the returned immutable row after dropping this fold.
    pub(crate) fn collect_observed_free_regions(&mut self) -> Result<Vec<RegionId>, &'static str> {
        self.validate_observation_owner()?;
        let Action::Regions(mapping) = &self.action else {
            return Err("observation owner changed");
        };
        let count = mapping.replacements.len();
        self.region_work(count.saturating_mul(2))?;
        let scratch = self.reserve_region_scratch(bounded::growth::<RegionId>(count))?;
        let result = (|| {
            let mut regions = Vec::new();
            regions
                .try_reserve_exact(count)
                .map_err(|_| "bounded observation output allocation failed")?;
            for region in self.observed_regions() {
                if region != RegionId::STATIC && region.bound_parts().is_none() {
                    regions.push(region);
                }
            }
            Ok(regions)
        })();
        self.release_region_scratch(scratch)?;
        result
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/observed_regions.rs"]
mod tests;
