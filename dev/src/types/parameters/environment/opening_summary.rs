//! Exact immutable callers may share structure when region opening is identity.
use super::{Environment, premises::TraitAssumptions};
use crate::{
    support::retained::Allocation,
    types::{RegionId, Store, arena::Identity, substitute::Bound},
};
use std::sync::{Arc, Weak};
#[cfg(test)]
#[path = "../../../../tests/invariants/free_callers.rs"]
mod tests;

pub(super) struct Summary {
    owner: Weak<Environment>,
    arena: Identity,
    free: Vec<RegionId>,
    outer: Vec<RegionId>,
    captured: bool,
    trait_only: bool,
}
impl Summary {
    fn current(&self, owner: &Arc<Environment>, types: &Store) -> bool {
        self.owner.ptr_eq(&Arc::downgrade(owner)) && types.has_identity(&self.arena)
    }
    pub(super) fn allocation(&self) -> Allocation {
        let mut total = Allocation::default();
        total.boxed(self);
        total.vector(&self.free);
        total.vector(&self.outer);
        // Include the Arc control block, arena token, and the allocation which
        // the weak owner can keep alive after its last strong reference drops.
        total.add(2 * size_of::<usize>() + Identity::allocation_bytes());
        total.add(size_of::<Environment>() + 2 * size_of::<usize>());
        total
    }
}
impl Environment {
    /// Reuse only a complete observation of this exact immutable caller/store.
    /// Every request still owns its fresh fold, restoration and proof debts.
    pub(crate) fn identity_region_opening(
        self: &Arc<Self>,
        types: &mut Store,
        fold: &mut Bound,
    ) -> Result<bool, &'static str> {
        self.identity_region_opening_observed::<false>(types, fold, &|_, _| Ok(()))
    }
    pub(crate) fn identity_region_opening_charged(
        self: &Arc<Self>,
        types: &mut Store,
        fold: &mut Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        self.identity_region_opening_observed::<true>(types, fold, charge)
    }
    fn identity_region_opening_observed<const CHARGED: bool>(
        self: &Arc<Self>,
        types: &mut Store,
        fold: &mut Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        fold.account_region_summary_work(1)?;
        let Some(traits) = &self.traits else {
            return Ok(false);
        };
        if traits.opening.get().is_none() {
            let mut observer = if CHARGED {
                Bound::observe_regions_charged(charge)?
            } else {
                Bound::observe_budgeted_regions(1_000_000, usize::MAX)?
            };
            let result = self.inspect_opening_observed::<CHARGED>(types, &mut observer, charge);
            // Transfer attempted work before propagating a failed inspection.
            fold.account_region_summary_work(observer.steps)?;
            let (free, outer, captured) = result?;
            if CHARGED {
                charge(
                    1,
                    size_of::<Summary>() + 2 * size_of::<usize>() + Identity::allocation_bytes(),
                )?;
            }
            let summary = Arc::new(Summary {
                owner: Arc::downgrade(self),
                arena: types.arena_identity(),
                free,
                outer,
                captured,
                trait_only: self.permits_trait_opening(),
            });
            // Only successful structural observations are retained. A fork may
            // share this table but cannot consume another environment's row.
            let _ = traits.opening.set(summary);
        }
        let summary = traits.opening.get().ok_or("missing opening summary")?;
        if !summary.current(self, types) || summary.captured {
            return Ok(false);
        }
        for &region in &summary.free {
            fold.account_region_summary_work(1)?;
            if fold.region_observed::<CHARGED>(region, 0, charge)? != region {
                return Err("identity opening changed an existing caller lifetime");
            }
        }
        Ok(true)
    }

    fn inspect_opening_observed<const CHARGED: bool>(
        &self,
        types: &mut Store,
        observer: &mut Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(Vec<RegionId>, Vec<RegionId>, bool), &'static str> {
        // The remaining components (parameters, const identities and projection
        // promises) contain no region-bearing payload. Inspect their presence,
        // rather than scanning closed parameter/name tables on every query.
        observer.account_region_summary_work(8)?;
        if let Some(assumptions) = &self.assumptions {
            for &ty in assumptions
                .predicates
                .keys()
                .chain(assumptions.sized.iter())
            {
                observer.account_region_summary_work(1)?;
                observer.apply_inside_observed::<CHARGED>(types, ty, 0, charge)?;
            }
        }
        if let Some(mapped) = self
            .traits
            .as_ref()
            .and_then(|traits| traits.mapped.as_ref())
        {
            mapped
                .index
                .opening()
                .ok_or("missing trait view observation")?
                .replay_observed::<CHARGED>(types, observer, charge)?;
        } else {
            for clause in self.trait_clauses()? {
                observer.account_region_summary_work(1)?;
                if CHARGED {
                    observer.clause_charged(types, clause, charge)?;
                } else {
                    observer.clause(types, clause)?;
                }
            }
        }
        for clause in self.projection_clauses() {
            observer.account_region_summary_work(1)?;
            if CHARGED {
                observer.clause_charged(types, clause, charge)?;
            } else {
                observer.clause(types, clause)?;
            }
        }
        if let Some(builtins) = &self.builtins {
            for clause in builtins.clauses() {
                observer.account_region_summary_work(1)?;
                if CHARGED {
                    observer.clause_charged(types, clause, charge)?;
                } else {
                    observer.clause(types, clause)?;
                }
            }
        }
        if let Some(regions) = &self.regions {
            regions.observe_regions_observed::<CHARGED>(observer, charge)?;
        }
        let count = observer.observed_regions().count();
        let sort_work = usize::try_from(usize::BITS - count.leading_zeros())
            .map_err(|_| "opening summary work count overflow")?;
        observer.account_region_summary_work(count.saturating_mul(sort_work + 2))?;
        if CHARGED {
            charge(count, count.saturating_mul(2 * size_of::<RegionId>()))?;
        }
        let mut free = Vec::new();
        free.try_reserve_exact(count)
            .map_err(|_| "opening summary allocation failed")?;
        let mut captured = self.body_aliases().is_some();
        let mut outer = Vec::new();
        for region in observer.observed_regions() {
            if region.bound_parts().is_some() {
                captured = true;
                if outer.capacity() == 0 {
                    outer
                        .try_reserve_exact(count)
                        .map_err(|_| "captured summary allocation failed")?;
                }
                outer.push(region);
            } else {
                free.push(region);
            }
        }
        free.sort_unstable_by_key(|region| region.0);
        outer.sort_unstable_by_key(|region| region.0);
        Ok((free, outer, captured))
    }

    fn permits_trait_opening(&self) -> bool {
        self.traits
            .as_ref()
            .is_some_and(|traits| traits.permits_opened_view())
            && self
                .assumptions
                .as_ref()
                .is_none_or(|part| !part.region_mapping)
            && self
                .builtins
                .as_ref()
                .is_none_or(|part| !part.region_mapping)
            && self
                .equalities
                .as_ref()
                .is_none_or(|part| !part.region_mapping)
            && self.regions.as_ref().is_none_or(|part| part.is_empty())
    }

    /// A fresh query owns the region map; only canonical clause buckets are
    /// shared. Captures absent from its actual request retain eager opening.
    pub(crate) fn mapped_trait_opening(
        self: &Arc<Self>,
        types: &mut Store,
        fold: &mut Bound,
    ) -> Result<Option<Arc<Self>>, &'static str> {
        self.mapped_trait_opening_observed::<false>(types, fold, &|_, _| Ok(()))
    }
    pub(crate) fn mapped_trait_opening_charged(
        self: &Arc<Self>,
        types: &mut Store,
        fold: &mut Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Option<Arc<Self>>, &'static str> {
        self.mapped_trait_opening_observed::<true>(types, fold, charge)
    }
    fn mapped_trait_opening_observed<const CHARGED: bool>(
        self: &Arc<Self>,
        types: &mut Store,
        fold: &mut Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Option<Arc<Self>>, &'static str> {
        use crate::types::parameters::premise_index::{ClauseIndex, Opening};
        fold.account_region_summary_work(1)?;
        let Some(traits) = &self.traits else {
            return Ok(None);
        };
        let Some(summary) = traits.opening.get() else {
            return Ok(None);
        };
        if !summary.current(self, types) || !summary.trait_only || summary.outer.is_empty() {
            return Ok(None);
        }
        for &region in &summary.outer {
            fold.account_region_summary_work(1)?;
            if fold.opened_region(region).is_none() {
                return Ok(None);
            }
        }
        for &region in &summary.free {
            fold.account_region_summary_work(1)?;
            fold.region_observed::<CHARGED>(region, 0, charge)?;
        }
        let mut work = 0;
        let result = if CHARGED {
            traits.index_charged(types, charge)
        } else {
            traits.index_counted(types, &mut work)
        };
        fold.account_region_summary_work(work)?;
        let index = result?;
        let count = fold
            .opened_region_count()
            .ok_or("missing actual query opening")?;
        let sort_work = usize::try_from(usize::BITS - count.leading_zeros())
            .map_err(|_| "trait opening work count overflow")?;
        fold.account_region_summary_work(count.saturating_mul(sort_work + 3).saturating_add(7))?;
        if CHARGED {
            charge(
                count,
                count.saturating_mul(size_of::<(RegionId, RegionId)>()),
            )?;
            charge(
                1,
                Opening::construction_bytes(count)
                    + ClauseIndex::opening_allocation_bytes()
                    + size_of::<TraitAssumptions>()
                    + size_of::<super::premises::Mapped>()
                    + size_of::<Self>()
                    + 6 * size_of::<usize>(),
            )?;
        }
        let mut regions = Vec::new();
        regions
            .try_reserve_exact(count)
            .map_err(|_| "trait query mapping allocation failed")?;
        regions.extend(fold.opened_regions());
        regions.sort_unstable_by_key(|(original, _)| original.0);
        let opening = Arc::new(Opening::new(Arc::clone(self), types, regions));
        let mut work = 0;
        let result = ClauseIndex::opened(index, opening, types, &mut work);
        fold.account_region_summary_work(work)?;
        let index = result?;
        let mut result = self.as_ref().clone();
        result.traits = Some(TraitAssumptions::mapped(Arc::clone(traits), index)?);
        // Eligibility permits only an empty equivalence graph here. The eager
        // projection fold likewise removes producer-only body alias metadata.
        result.regions = None;
        result.region_mapping = true;
        Ok(Some(Arc::new(result)))
    }

    /// Structural cache bytes are separate from proof/index ownership. The row
    /// authenticates its own owner, so foreign/forked rows report no usable cache.
    #[cfg(test)]
    pub(crate) fn opening_summary_allocation(self: &Arc<Self>, types: &Store) -> Allocation {
        self.traits
            .as_ref()
            .and_then(|traits| traits.opening.get())
            .filter(|summary| summary.current(self, types))
            .map_or_else(Allocation::default, |summary| summary.allocation())
    }
}

impl Environment {
    /// Preserve immutable canonical partitions when opening one proof request.
    /// Only the genuinely non-shareable regional partitions are copied.
    pub(crate) fn opened_regions_charged(
        self: &Arc<Self>,
        types: &mut Store,
        fold: &mut Bound,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Arc<Self>, &'static str> {
        if !self.requires_region_mapping()
            || self.identity_region_opening_charged(types, fold, charge)?
        {
            return Ok(Arc::clone(self));
        }
        if let Some(mapped) = self.mapped_trait_opening_charged(types, fold, charge)? {
            return Ok(mapped);
        }
        charge(1, size_of::<Self>() + 2 * size_of::<usize>())?;
        self.substitute_declared_regions_charged(types, fold, charge)
            .map(Arc::new)
    }
}
