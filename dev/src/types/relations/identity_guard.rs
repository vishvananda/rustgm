//! Conditional equality compares complete typed inputs without constraining inference.
use super::{Argument, Constraints, Error, Kind, Region, RegionId, Relation, Scope, TypeId};
use crate::types::{GenericArgs, outlives::Projection, region_graph::VerificationId};
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
    fn arguments(&mut self, a: GenericArgs, b: GenericArgs, depth: u16) -> Result<bool, Error> {
        let a = self.proof.types.arguments(a).ok_or(Error::Identity)?;
        let b = self.proof.types.arguments(b).ok_or(Error::Identity)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.zip(b) {
            self.charge(1)?;
            match (a, b) {
                (Argument::Type(a), Argument::Type(b)) => self.child(a, b, depth)?,
                (Argument::Lifetime(a), Argument::Lifetime(b)) => {
                    if !self.regions(a, b, depth)? {
                        return Ok(false);
                    }
                }
                (Argument::Const(a), Argument::Const(b)) if a == b => {}
                _ => return Ok(false),
            }
        }
        Ok(true)
    }
    fn list(
        &mut self,
        a: crate::types::ListId,
        b: crate::types::ListId,
        depth: u16,
    ) -> Result<bool, Error> {
        let a = self.proof.types.list(a).ok_or(Error::Identity)?;
        let b = self.proof.types.list(b).ok_or(Error::Identity)?;
        if a.len() != b.len() {
            return Ok(false);
        }
        for (&a, &b) in a.iter().zip(b) {
            self.child(a, b, depth)?;
        }
        Ok(true)
    }
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive structural dispatch keeps conditional identity distinct from coercion"
    )]
    fn compare(&mut self, a: TypeId, b: TypeId, depth: u16) -> Result<bool, Error> {
        let a = self.proof.types.kind(a).ok_or(Error::Identity)?;
        let b = self.proof.types.kind(b).ok_or(Error::Identity)?;
        match (a, b) {
            (Kind::Never, Kind::Never) => {}
            (Kind::Primitive(a), Kind::Primitive(b)) if a == b => {}
            (Kind::Parameter(a), Kind::Parameter(b)) if a == b => {}
            (
                Kind::Reference {
                    region: a,
                    mutable: x,
                    referent: left,
                },
                Kind::Reference {
                    region: b,
                    mutable: y,
                    referent: right,
                },
            ) if x == y => {
                if !self.regions(a, b, depth)? {
                    return Ok(false);
                }
                self.child(left, right, depth)?;
            }
            (
                Kind::Pointer {
                    mutable: x,
                    pointee: a,
                },
                Kind::Pointer {
                    mutable: y,
                    pointee: b,
                },
            ) if x == y => self.child(a, b, depth)?,
            (
                Kind::Array {
                    length: x,
                    element: a,
                },
                Kind::Array {
                    length: y,
                    element: b,
                },
            ) if x == y => self.child(a, b, depth)?,
            (
                Kind::ArrayTerm {
                    length: x,
                    element: a,
                },
                Kind::ArrayTerm {
                    length: y,
                    element: b,
                },
            ) if x == y => self.child(a, b, depth)?,
            (Kind::Slice(a), Kind::Slice(b)) => self.child(a, b, depth)?,
            (Kind::Tuple(a), Kind::Tuple(b)) => return self.list(a, b, depth),
            (Kind::Adt(x, a), Kind::Adt(y, b)) if x == y => return self.arguments(a, b, depth),
            (
                Kind::Function {
                    bound_regions: x,
                    parameters: a,
                    result: ar,
                    unsafe_: au,
                },
                Kind::Function {
                    bound_regions: y,
                    parameters: b,
                    result: br,
                    unsafe_: bu,
                },
            ) if x == y && au == bu => {
                let depth = depth.checked_add(1).ok_or(Error::Budget)?;
                self.child(ar, br, depth)?;
                return self.list(a, b, depth);
            }
            (
                Kind::FunctionItem {
                    item: x,
                    arguments: a,
                    signature: ar,
                },
                Kind::FunctionItem {
                    item: y,
                    arguments: b,
                    signature: br,
                },
            ) if x == y => {
                self.child(ar, br, depth)?;
                return self.arguments(a, b, depth);
            }
            (Kind::Projection(a), Kind::Projection(b)) if a.item == b.item => {
                let at = self
                    .proof
                    .types
                    .trait_application(a.trait_application)
                    .ok_or(Error::Identity)?;
                let bt = self
                    .proof
                    .types
                    .trait_application(b.trait_application)
                    .ok_or(Error::Identity)?;
                if at.item != bt.item {
                    return Ok(false);
                }
                self.child(a.subject, b.subject, depth)?;
                return Ok(self.arguments(at.arguments, bt.arguments, depth)?
                    && self.arguments(a.arguments, b.arguments, depth)?);
            }
            _ => return Ok(false),
        }
        Ok(true)
    }
}
