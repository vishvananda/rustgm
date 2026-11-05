//! Complete projection inputs support verification and local method lifetime choices.
use super::{Constraints, Error, Region, RegionId, Relation, Scope, TypeId, structural::Compare};
use crate::support::hash::HashSet;
use crate::types::{outlives::Projection, region_graph::VerificationId};

#[derive(Clone, Copy)]
pub(super) struct View {
    pub projection: Projection,
    pub scope: Scope,
}
struct Match<'a, 't, const CHOOSE: bool> {
    proof: &'a mut Constraints<'t>,
    left: View,
    right: View,
    work: Vec<(TypeId, TypeId, u16)>,
    seen: HashSet<(TypeId, TypeId, u16)>,
    guards: Vec<VerificationId>,
    source: u32,
}
impl Constraints<'_> {
    /// Equal complete inputs suffice for opaque result identity. Their equality
    /// is a condition to verify, never an equation inferred from equal results.
    pub(super) fn relate_projection<const METHOD: bool>(
        &mut self,
        relation: Relation,
        source: u32,
    ) -> Result<(), Error> {
        let view = |ty, scope| View {
            projection: Projection { ty, depth: 0 },
            scope,
        };
        let guards = self
            .projection_inputs::<METHOD>(
                view(relation.actual, relation.left),
                view(relation.expected, relation.right),
                source,
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
        self.projection_inputs::<false>(left, right, 0)
    }

    fn projection_inputs<const CHOOSE: bool>(
        &mut self,
        left: View,
        right: View,
        source: u32,
    ) -> Result<Option<Vec<VerificationId>>, Error> {
        let mut query = Match::<CHOOSE> {
            proof: self,
            left,
            right,
            work: vec![(left.projection.ty, right.projection.ty, 0)],
            seen: HashSet::default(),
            guards: Vec::new(),
            source,
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
impl<const CHOOSE: bool> Match<'_, '_, CHOOSE> {
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
            if CHOOSE {
                self.proof.graph.require(a, b, self.source)?;
                self.proof.graph.require(b, a, self.source)?;
            } else {
                self.guards.push(self.proof.graph.verify_outlives(a, b)?);
                self.guards.push(self.proof.graph.verify_outlives(b, a)?);
            }
        }
        Ok(true)
    }
}
impl<'t, const CHOOSE: bool> super::structural::Compare<'t> for Match<'_, 't, CHOOSE> {
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
