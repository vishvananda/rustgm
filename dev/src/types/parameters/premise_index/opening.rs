//! One query maps consumed canonical premises; the shared index grants no proof.
use super::super::{environment::Environment, predicates::Predicate};
use crate::{
    support::{hash::HashMap, retained::Allocation},
    types::{RegionId, Store, arena::Identity, infer, substitute::Bound},
};
use std::sync::{Arc, Mutex};

pub(crate) struct Opening {
    source: Arc<Environment>,
    arena: Identity,
    regions: Vec<(RegionId, RegionId)>,
    fold: Mutex<Bound>,
}
impl Opening {
    pub(in crate::types::parameters) fn construction_bytes(count: usize) -> usize {
        size_of::<Self>()
            + 2 * size_of::<usize>()
            + Identity::allocation_bytes()
            + Bound::region_mapping_bytes()
            + crate::support::retained::new_map::<RegionId, RegionId>(count)
    }

    pub(in crate::types::parameters) fn require_base(
        &self,
        types: &Store,
        index: &Arc<super::ClauseIndex>,
        work: &mut usize,
    ) -> Result<(), &'static str> {
        *work = work.saturating_add(1);
        self.validate(types)?;
        let actual = self
            .source
            .trait_index_counted(types, work)?
            .ok_or("trait opening lost its actual canonical index")?;
        if !Arc::ptr_eq(&actual, index) {
            return Err("trait opening has a foreign canonical caller index");
        }
        Ok(())
    }

    pub(in crate::types::parameters) fn new(
        source: Arc<Environment>,
        types: &mut Store,
        regions: Vec<(RegionId, RegionId)>,
    ) -> Self {
        let replacements: HashMap<_, _> = regions.iter().copied().collect();
        Self {
            source,
            arena: types.arena_identity(),
            regions,
            fold: Mutex::new(Bound::restore_regions(replacements)),
        }
    }
    pub(in crate::types::parameters) fn validate(&self, types: &Store) -> Result<(), &'static str> {
        if !types.has_identity(&self.arena) {
            return Err("trait opening belongs to a different type arena");
        }
        if self.source.has_trait_opening() {
            return Err("trait opening requires its original canonical caller");
        }
        Ok(())
    }
    pub(crate) fn predicate(
        &self,
        types: &mut Store,
        context: &mut infer::Context,
        predicate: Predicate,
    ) -> Result<Predicate, infer::Error> {
        context.account_work(1)?;
        let mut work = 0;
        let result = self.transform(types, predicate, &mut work);
        context.account_work(work)?;
        result
    }
    pub(in crate::types::parameters) fn substitute(
        &self,
        types: &mut Store,
        predicate: Predicate,
        work: &mut usize,
    ) -> Result<Predicate, &'static str> {
        self.transform(types, predicate, work)
            .map_err(|error| match error {
                infer::Error::Budget => "mapped trait substitution work exhausted",
                _ => "invalid mapped trait substitution",
            })
    }

    fn transform(
        &self,
        types: &mut Store,
        predicate: Predicate,
        work: &mut usize,
    ) -> Result<Predicate, infer::Error> {
        *work = work.saturating_add(1);
        self.validate(types)
            .map_err(|_| infer::Error::InvalidIdentity)?;
        let mut fold = self
            .fold
            .lock()
            .map_err(|_| infer::Error::InvalidIdentity)?;
        let before = fold.steps;
        let result = fold.predicate(types, predicate);
        *work = work.saturating_add(fold.steps.saturating_sub(before));
        result.map_err(|_| {
            if fold.steps > 1_000_000 {
                infer::Error::Budget
            } else {
                infer::Error::InvalidIdentity
            }
        })
    }
    pub(in crate::types::parameters) fn replay_observed<const CHARGED: bool>(
        &self,
        types: &Store,
        fold: &mut Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        fold.account_region_summary_work(1)?;
        self.validate(types)?;
        for &(_, region) in &self.regions {
            fold.account_region_summary_work(1)?;
            if fold.region_observed::<CHARGED>(region, 0, charge)? != region {
                return Err("opened trait caller changed its captured lifetime");
            }
        }
        Ok(())
    }
    pub(crate) fn allocation(&self) -> Allocation {
        let mut total = Allocation::default();
        total.boxed(self);
        total.vector(&self.regions);
        total.add(2 * size_of::<usize>() + Identity::allocation_bytes());
        // The original environment remains immutable input, shared with its
        // source owner. Its bytes are not copied into each query-local view.
        let fold = match self.fold.lock() {
            Ok(fold) => fold,
            Err(poisoned) => poisoned.into_inner(),
        };
        let (bytes, visits) = fold.retained_allocation();
        total.include(Allocation { bytes, visits });
        total
    }
}
