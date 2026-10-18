//! Conditional equality compares complete typed inputs without constraining inference.
use super::{Constraints, Error, Region, RegionId, Relation, Scope, TypeId, structural::Compare};
use crate::types::{outlives::Projection, region_graph::VerificationId};
use std::collections::HashSet;

#[derive(Clone, Copy)]
pub(super) struct View {
    pub projection: Projection,
    pub scope: Scope,
}
struct Match<'a, 't> {
    proof: &'a mut Constraints<'t>,
    left: View,
    right: View,
    work: Vec<(TypeId, TypeId, u16)>,
    seen: HashSet<(TypeId, TypeId, u16)>,
    guards: Vec<VerificationId>,
}
impl Constraints<'_> {
    /// Equal complete inputs suffice for opaque result identity. Their equality
    /// is a condition to verify, never an equation inferred from equal results.
    pub(super) fn relate_projection(
        &mut self,
        relation: Relation,
        source: u32,
    ) -> Result<(), Error> {
        let view = |ty, scope| View {
            projection: Projection { ty, depth: 0 },
            scope,
        };
        let guards = self
            .projection_guards(
                view(relation.actual, relation.left),
                view(relation.expected, relation.right),
            )?
            .ok_or(Error::Identity)?;
        if !guards.is_empty() {
            let condition = self.graph.verify_all(&guards)?;
            self.graph.require_verification(condition, source)?;
        }
        Ok(())
    }
    pub(super) fn projection_guards(
        &mut self,
        left: View,
        right: View,
    ) -> Result<Option<Vec<VerificationId>>, Error> {
        let mut query = Match {
            proof: self,
            left,
            right,
            work: vec![(left.projection.ty, right.projection.ty, 0)],
            seen: HashSet::new(),
            guards: Vec::new(),
        };
        while let Some((a, b, depth)) = query.work.pop() {
            query.charge(1)?;
            if (a == b
                && (left.projection.depth == right.projection.depth && left.scope == right.scope
                    || query.proof.types.escaping_binders(a) == Some(0)))
                || !query.seen.insert((a, b, depth))
            {
                continue;
            }
            if !query.compare(a, b, depth)? {
                return Ok(None);
            }
        }
        Ok(Some(query.guards))
    }
}
impl Match<'_, '_> {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.proof.steps = self.proof.steps.saturating_add(count);
        if self.proof.steps.saturating_add(self.work.len()) > self.proof.limit {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn child(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<(), Error> {
        self.charge(1)?;
        self.work.push((a, b, depth));
        Ok(())
    }
    fn resolve(&self, region: RegionId, view: View, nested: u16) -> Result<Region, Error> {
        let region = if let Some((binder, slot)) = region.bound_parts() {
            let offset = nested
                .checked_add(view.projection.depth)
                .ok_or(Error::Budget)?;
            RegionId::bound(binder.checked_sub(offset).ok_or(Error::Identity)?, slot)
                .map_err(|_| Error::Identity)?
        } else {
            region
        };
        self.proof.region(region, view.scope)
    }
    fn regions(&mut self, a: RegionId, b: RegionId, nested: u16) -> Result<bool, Error> {
        let local = |r: RegionId| r.bound_parts().is_some_and(|(binder, _)| binder < nested);
        if local(a) || local(b) {
            return Ok(a == b);
        }
        let a = self.resolve(a, self.left, nested)?;
        let b = self.resolve(b, self.right, nested)?;
        if a != b {
            self.charge(2)?;
            self.guards.push(self.proof.graph.verify_outlives(a, b)?);
            self.guards.push(self.proof.graph.verify_outlives(b, a)?);
        }
        Ok(true)
    }
}
impl<'t> super::structural::Compare<'t> for Match<'_, 't> {
    fn types(&self) -> &'t crate::types::Store {
        self.proof.types
    }
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        Match::charge(self, count)
    }
    fn child(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<(), Error> {
        Match::child(self, a, b, depth)
    }
    fn regions(&mut self, a: RegionId, b: RegionId, depth: u16) -> Result<bool, Error> {
        Match::regions(self, a, b, depth)
    }
}
