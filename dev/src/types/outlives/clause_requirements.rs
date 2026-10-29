//! Stream independent clause-formation debts once per body, preserving their first source.
use super::{
    Bounds, ComponentKind, Projection, RegionId, Requirements, ScalarRequirements, Scan, Store,
    TypeId,
    component_sets::{Atom, Sets},
    components::Walk,
    requirements::Demand,
};
use crate::types::{bound_uses::captures::Slots, callable::Templates};
#[cfg(test)]
#[path = "../../../tests/invariants/body_formation_work.rs"]
mod tests;

pub(crate) struct Formation<'a> {
    types: &'a Store,
    bounds: &'a Bounds,
    callables: &'a Templates,
    scan: Scan<'static>,
    slots: Slots,
    demand: Deferred,
}
impl<'a> Formation<'a> {
    pub(super) fn new(types: &'a Store, bounds: &'a Bounds, callables: &'a Templates) -> Self {
        Self {
            types,
            bounds,
            callables,
            scan: Scan {
                families: bounds.has_family_inputs(),
                ..Scan::default()
            },
            slots: Slots::default(),
            demand: Deferred::default(),
        }
    }
    /// Return only new debts reached from this actual source root. Sharing is
    /// valid within one body; formation never supplies an assumption to its proof.
    pub(crate) fn requirements(
        &mut self,
        ty: TypeId,
        bound_regions: u32,
    ) -> Result<Requirements<'_>, &'static str> {
        self.scan.budget.charge()?;
        if bound_regions > u32::from(u16::MAX) + 1 {
            return Err("too many clause formation lifetime slots");
        }
        let before = self.slots.steps;
        let slots = self.slots.collect(self.types, ty)?;
        self.scan
            .budget
            .work(self.slots.steps.saturating_sub(before))?;
        if slots > bound_regions {
            return Err("clause formation references an unbound lifetime");
        }
        self.scan.pairs.clear();
        self.scan.parameter_pairs.clear();
        self.scan.projection_pairs.clear();
        self.scan.uses.clear();
        self.scan.family_uses.clear();
        self.scan.visit_with(self.types, ty, 1, &mut self.demand)?;
        self.scan
            .demands(self.types, self.bounds, self.callables, &mut self.demand)?;
        Ok(Requirements {
            scalars: ScalarRequirements {
                regions: &self.scan.pairs,
                parameters: &self.scan.parameter_pairs,
            },
            projections: &self.scan.projection_pairs,
        })
    }
}

/// Share structural component sets while retaining target-specific debt publication.
enum Deferred {
    Single {
        key: Option<(RegionId, ComponentKind)>,
        walk: Walk,
    },
    Shared(Box<Sets>),
}
impl Default for Deferred {
    fn default() -> Self {
        Self::Single {
            key: None,
            walk: Walk::default(),
        }
    }
}
impl Deferred {
    fn record(
        scan: &mut Scan<'_>,
        shorter: RegionId,
        atoms: impl Iterator<Item = Atom>,
    ) -> Result<(), &'static str> {
        for atom in atoms {
            scan.budget.charge()?;
            match atom {
                Atom::Region(longer) => scan.record((longer, shorter))?,
                Atom::Parameter(parameter) => {
                    let pair = (parameter, shorter);
                    if scan.unique_parameters.insert(pair) {
                        scan.parameter_pairs.push(pair);
                    }
                }
                Atom::Projection(projection) => {
                    let pair = (projection, shorter);
                    if scan.unique_projections.insert(pair) {
                        scan.projection_pairs.push(pair);
                    }
                }
            }
        }
        Ok(())
    }
}
impl Demand for Deferred {
    fn require(
        &mut self,
        scan: &mut Scan<'_>,
        types: &Store,
        ty: TypeId,
        shorter: RegionId,
        depth: u16,
        kind: ComponentKind,
    ) -> Result<(), &'static str> {
        // Defer the whole debt BEFORE decomposing its components. Child type
        // formation is visited independently, retaining nested independent debts.
        if scan
            .components
            .captures(types, ty, depth, &mut scan.budget)?
        {
            return Ok(());
        }
        let requested = (shorter, kind);
        if let Self::Single { key: Some(key), .. } = self
            && *key != requested
        {
            // One cursor is linear for a single target/policy. Promote once when
            // sharing across targets becomes necessary; published debt identities
            // stay in Scan, preserving the first source even after this rebuild.
            *self = Self::Shared(Box::default());
        }
        match self {
            Self::Single { key, walk } => {
                *key = Some(requested);
                let (regions, parameters, projections) =
                    walk.extend(types, &[ty], depth, kind, &mut scan.budget)?;
                Self::record(
                    scan,
                    shorter,
                    regions
                        .into_iter()
                        .map(Atom::Region)
                        .chain(parameters.into_iter().map(Atom::Parameter))
                        .chain(projections.into_iter().map(Atom::Projection)),
                )
            }
            Self::Shared(sets) => {
                let atoms = sets.extend(types, ty, depth, kind, shorter, &mut scan.budget)?;
                Self::record(scan, shorter, atoms.into_iter())
            }
        }
    }
    fn projection(
        &mut self,
        scan: &mut Scan<'_>,
        types: &Store,
        projection: Projection,
        shorter: RegionId,
    ) -> Result<(), &'static str> {
        self.require(
            scan,
            types,
            projection.ty,
            shorter,
            projection.depth,
            ComponentKind::TypeBound,
        )
    }
}
