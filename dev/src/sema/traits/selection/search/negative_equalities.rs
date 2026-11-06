//! Closed automatic negatives may pay private equations without assuming caller relations.
use super::{Declarations, Predicate, Search, Source};
use crate::{
    resolve::model::DefId,
    support::{
        hash::{HashMap, HashSet},
        retained::Growth,
    },
    types::{RegionId, auto_traits::AutoTrait, lifetime_proof::ScopeId, relations::Constraints},
};
use std::ops::Range;

fn custom(facts: &Declarations, goal: Predicate) -> Result<bool, String> {
    if !super::automatic_traits::predicate(facts, goal)? {
        return Ok(false);
    }
    let (application, _) = goal
        .bound
        .trait_parts()
        .ok_or("missing automatic application")?;
    let item = facts
        .types
        .trait_application(application)
        .ok_or("missing automatic goal")?
        .item;
    if let Some(catalog) = facts.source_bindings.as_deref() {
        for marker in [
            AutoTrait::Freeze,
            AutoTrait::Send,
            AutoTrait::Sync,
            AutoTrait::Unpin,
            AutoTrait::UnsafeUnpin,
        ] {
            let role = super::freezing::role(marker).ok_or("missing source automatic role")?;
            if catalog
                .language(role)
                .is_some_and(|d| d.definition == DefId(item.0))
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

struct Equations {
    private: Range<u32>,
    regions: HashMap<RegionId, RegionId>,
    private_count: u32,
    rigid_count: u32,
    pairs: Vec<(RegionId, RegionId)>,
}
impl Equations {
    fn new(private: Range<u32>) -> Self {
        Self {
            private,
            regions: HashMap::default(),
            private_count: 0,
            rigid_count: 0,
            pairs: Vec::new(),
        }
    }
    fn limit(&self) -> usize {
        let extent = (self.private_count as usize)
            .saturating_add(self.rigid_count as usize)
            .saturating_add(self.pairs.len())
            .saturating_add(2);
        extent.saturating_mul(extent).saturating_mul(8)
    }
    fn region(
        &mut self,
        region: RegionId,
        pay: &mut impl FnMut(Growth) -> Result<(), String>,
    ) -> Result<RegionId, String> {
        if region == RegionId::STATIC {
            return Ok(region);
        }
        let owned = self.private.contains(&region.0);
        if !owned
            && (region.bound_parts().is_some_and(|(depth, _)| depth != 0)
                || (region.bound_parts().is_none() && region.0 >= self.private.start))
        {
            return Err("negative equality refers to an unowned lifetime".into());
        }
        if let Some(&existing) = self.regions.get(&region) {
            return Ok(existing);
        }
        pay(Growth::map::<RegionId, RegionId>(
            self.regions.len(),
            self.regions.capacity(),
            1,
        ))?;
        let replacement = if owned {
            let next = self
                .private_count
                .checked_add(1)
                .ok_or("negative private lifetime overflow")?;
            let local = RegionId(next);
            if local.bound_parts().is_some() {
                return Err("negative private lifetime overflow".into());
            }
            self.private_count = next;
            local
        } else {
            let slot =
                u16::try_from(self.rigid_count).map_err(|_| "negative caller lifetime overflow")?;
            let local = RegionId::bound(0, slot)?;
            self.rigid_count = self
                .rigid_count
                .checked_add(1)
                .ok_or("negative caller lifetime overflow")?;
            local
        };
        self.regions.insert(region, replacement);
        Ok(replacement)
    }
    fn pay(self, facts: &Declarations, limit: usize) -> Result<bool, String> {
        let mut constraints = Constraints::new(
            &facts.types,
            &facts.lifetimes.variances,
            self.rigid_count,
            limit,
        )
        .map_err(|e| e.to_string())?;
        for region in 1..=self.private_count {
            constraints
                .variable(RegionId(region))
                .map_err(|e| e.to_string())?;
        }
        for (a, b) in self.pairs {
            constraints.outlives(a, b, 0).map_err(|e| e.to_string())?;
            constraints.outlives(b, a, 0).map_err(|e| e.to_string())?;
        }
        match constraints.check_aliases() {
            Ok(_) => Ok(true),
            Err(crate::types::region_graph::Error::Unsatisfied { .. }) => Ok(false),
            Err(error) => Err(error.to_string()),
        }
    }
}

impl Search<'_> {
    fn negative_growth(&mut self, growth: Growth) -> Result<(), String> {
        self.account_query_work(growth.bytes.saturating_add(growth.work).saturating_add(1))
    }
    pub(super) fn pay_negative_equalities(
        &mut self,
        facts: &Declarations,
        root: usize,
    ) -> Result<bool, String> {
        let node = self
            .query
            .nodes
            .get(root)
            .ok_or("missing negative proof root")?;
        if node.debt_free || !custom(facts, node.goal)? {
            return Ok(false);
        }
        let Some(private) = self.query.allocations.plain() else {
            return Ok(false);
        };
        if self.query.scopes.is_some() || private.start != self.caller.first_region {
            return Ok(false);
        }
        let Some(Source::Implementation { owner, arguments }) = node.source else {
            return Err("negative equality has no implementation owner".into());
        };
        if !super::super::refutation::implementation_negative(facts, owner)? {
            return Err("negative equality owner has positive polarity".into());
        }
        self.negative_growth(Growth::vector::<usize>(0, 0, 1))?;
        let mut equations = Equations::new(private);
        let mut work = vec![root];
        let mut seen = HashSet::default();
        while let Some(index) = work.pop() {
            self.account_query_work(1)?;
            if seen.contains(&index) {
                continue;
            }
            self.negative_growth(Growth::map::<usize, ()>(seen.len(), seen.capacity(), 1))?;
            seen.insert(index);
            let node = self
                .query
                .nodes
                .get(index)
                .ok_or("missing negative proof child")?;
            if node.provisional_size
                || node.cycle_dependent
                || self.query.node_scope(index) != ScopeId::ROOT
                || !node.outlives.is_empty()
                || !node.type_outlives.is_empty()
                || !node.sized.is_empty()
            {
                return Ok(false);
            }
            if node.first < equations.private.start
                || node.end > equations.private.end
                || node.end < node.first
            {
                return Err("negative proof has a foreign private interval".into());
            }
            if index != root {
                super::super::refutation::require_positive_source(facts, node.source.as_ref())?;
            } else if !matches!(node.source, Some(Source::Implementation { owner: actual, arguments: args }) if actual == owner && args == arguments)
            {
                return Err("negative equality application changed".into());
            }
            let count = node
                .children
                .len()
                .saturating_add(node.equalities.len().saturating_mul(2));
            let mut growth =
                Growth::vector::<usize>(work.len(), work.capacity(), node.children.len());
            growth.include(Growth::vector::<(RegionId, RegionId)>(
                equations.pairs.len(),
                equations.pairs.capacity(),
                node.equalities.len(),
            ));
            let pairs = node.equalities.len();
            self.negative_growth(growth)?;
            for pair in 0..pairs {
                let (a, b) = self.query.nodes[index].equalities[pair];
                let mut pay = |growth| self.negative_growth(growth);
                let a = equations.region(a, &mut pay)?;
                let b = equations.region(b, &mut pay)?;
                equations.pairs.push((a, b));
            }
            work.extend(self.query.nodes[index].children.iter().copied());
            self.account_query_work(count)?;
        }
        // Prepay the bounded graph's worst-case propagation as well as its live
        // input/output storage. Failure keeps this payment in the same query.
        let limit = equations.limit();
        self.account_query_work(limit)?;
        equations.pay(facts, limit)
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/automatic_negative_equalities.rs"]
mod tests;
