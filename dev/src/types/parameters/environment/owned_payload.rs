//! A mapped environment charges copied payload, excluding unchanged declaration tables.
use super::Environment;
use crate::{
    support::retained::{Allocation, Growth},
    types::parameters::premise_index,
};
use std::sync::Arc;
#[cfg(test)]
#[path = "../../../../tests/invariants/alias_accounting.rs"]
pub(crate) mod alias_tests;
#[derive(Default)]
pub(crate) struct Retained {
    base: bool,
    index: premise_index::Retained,
}
impl Environment {
    /// Cost only for an actual declaration without prior regional payload.
    /// Existing region tables, even empty ones, need their separately sized clone.
    pub(crate) fn neutral_alias_finish_bound(&self, count: usize) -> Result<Growth, &'static str> {
        if self.builtins.is_some() || self.regions.is_some() {
            return Err("neutral alias finish has preexisting regional payload");
        }
        let mut growth = crate::types::relations::BodyAliases::construction_bound(count)?;
        growth.work = growth.work.saturating_add(2);
        if count != 0 {
            growth.bytes = growth
                .bytes
                .saturating_add(size_of::<Self>())
                .saturating_add(size_of::<crate::types::relations::RegionEqualities>())
                .saturating_add(4 * size_of::<usize>());
        }
        Ok(growth)
    }
    /// Reconcile only the fresh alias finish; other declaration Arcs stay shared.
    pub(crate) fn neutral_alias_owned_from(
        &self,
        shared: &Self,
    ) -> Result<Allocation, &'static str> {
        if shared.builtins.is_some() || shared.regions.is_some() {
            return Err("neutral alias finish changed its original regional payload");
        }
        let regions = self
            .regions
            .as_deref()
            .filter(|regions| regions.is_empty() && !regions.has_equation_storage())
            .ok_or("neutral alias finish changed its actual equality construction")?;
        let aliases = regions
            .body_aliases()
            .ok_or("neutral alias finish omitted its actual retained ledger")?;
        let mut retained = self.retain_owned_from(shared, &mut Retained::default());
        retained.add(6 * size_of::<usize>());
        retained.add(aliases.control_allocation());
        Ok(retained)
    }

    /// Accounting only for already validated trait predicates whose
    /// subject/arguments are rigid parameters or scalar leaves. Wider clauses
    /// must use an explicit wider estimator before any environment allocation.
    pub(crate) fn trait_leaf_allocation_bound(
        parameters: usize,
        clauses: usize,
        roots: usize,
    ) -> usize {
        use crate::support::retained::new_map;
        use crate::types::{
            TypeId,
            parameters::{ParameterId, uses::Uses},
        };
        let retained = size_of::<Self>()
            .saturating_add(4 * size_of::<usize>())
            .saturating_add(size_of::<super::HashMap<ParameterId, super::Abilities>>())
            .saturating_add(new_map::<ParameterId, super::Abilities>(parameters))
            .saturating_add(super::premises::TraitAssumptions::leaf_allocation_bound(
                clauses,
            ));
        let scratch = size_of::<Uses>()
            .saturating_add(size_of::<Self>() + 2 * size_of::<usize>())
            .saturating_add(size_of::<super::HashMap<ParameterId, super::Abilities>>())
            .saturating_add(new_map::<ParameterId, ()>(roots))
            .saturating_add(new_map::<TypeId, ()>(roots))
            .saturating_add(new_map::<crate::types::traits::ApplicationId, ()>(clauses))
            .saturating_add(roots.saturating_mul(4 * size_of::<TypeId>()));
        retained.saturating_add(scratch)
    }

    /// Preflight only a validated region-free scalar/parameter/function graph
    /// with Copy/Sized clauses. This is allocation accounting, not validation
    /// or permission to interpret an arbitrary predicate as a builtin.
    pub(crate) fn builtin_allocation_bound(
        parameters: usize,
        clauses: usize,
        nodes: usize,
        edges: usize,
    ) -> usize {
        use crate::support::retained::new_map;
        use crate::types::{
            TypeId,
            parameters::{ParameterId, uses::Uses},
        };
        let parameters = new_map::<ParameterId, super::Abilities>(parameters);
        let ordinary = size_of::<super::Assumptions>()
            .saturating_add(2 * size_of::<usize>())
            .saturating_add(new_map::<TypeId, super::Abilities>(clauses))
            .saturating_add(new_map::<TypeId, ()>(clauses));
        let quantified =
            super::permissions::Index::callable_allocation_bound(clauses, nodes, edges);
        // Uses shares one memoized graph walk across all clauses. No trait or
        // const application is possible in this deliberately narrow mode.
        let scratch = size_of::<Uses>()
            .saturating_add(new_map::<ParameterId, ()>(nodes))
            .saturating_add(new_map::<TypeId, ()>(nodes))
            .saturating_add(
                edges
                    .saturating_add(nodes)
                    .saturating_mul(4 * size_of::<TypeId>()),
            );
        size_of::<Self>()
            .saturating_add(
                4 * size_of::<usize>() + size_of::<super::HashMap<ParameterId, super::Abilities>>(),
            )
            .saturating_add(parameters)
            .saturating_add(ordinary)
            .saturating_add(quantified)
            .saturating_add(scratch)
    }
    pub(crate) fn retain_owned_from(&self, shared: &Self, observer: &mut Retained) -> Allocation {
        let mut total = Allocation::default();
        if !observer.base {
            observer.base = true;
            total.boxed(self);
            if !Arc::ptr_eq(&self.parameters, &shared.parameters) {
                total.boxed(self.parameters.as_ref());
                total.map(&self.parameters);
            }
            if let Some(value) = fresh(self.assumptions.as_ref(), shared.assumptions.as_ref()) {
                total.boxed(value);
                total.map(&value.predicates);
                total.set(&value.sized);
            }
            if let Some(value) = fresh(self.builtins.as_ref(), shared.builtins.as_ref()) {
                total.boxed(value);
                total.include(value.allocation());
            }
            if let Some(value) = fresh(self.regions.as_ref(), shared.regions.as_ref()) {
                total.boxed(value);
                total.include(value.allocation());
            }
            if let Some(value) = fresh(self.traits.as_ref(), shared.traits.as_ref()) {
                total.boxed(value);
                total.include(value.allocation());
            }
            if let Some(value) = fresh(self.equalities.as_ref(), shared.equalities.as_ref()) {
                total.boxed(value);
                total.include(value.allocation());
            }
            if let Some(value) = fresh(
                self.projection_promises.as_ref(),
                shared.projection_promises.as_ref(),
            ) {
                total.boxed(value);
                total.map(value);
            }
        }
        total.visits = total.visits.saturating_add(1);
        if let Some(value) = fresh(self.traits.as_ref(), shared.traits.as_ref()) {
            total.include(value.retain_index(&mut observer.index));
        }
        total
    }
}
fn fresh<'a, T>(value: Option<&'a Arc<T>>, shared: Option<&Arc<T>>) -> Option<&'a T> {
    value
        .filter(|value| shared.is_none_or(|shared| !Arc::ptr_eq(value, shared)))
        .map(Arc::as_ref)
}
