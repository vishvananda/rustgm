//! Stream independent clause-formation debts once per body, preserving their first source.
use super::{
    Bounds, ComponentKind, HashMap, Projection, RegionId, Requirements, ScalarRequirements, Scan,
    Store, TypeId, components::Walk, requirements::Demand,
};
use crate::types::{bound_uses::captures::Slots, callable::Templates};
#[cfg(test)]
#[path = "../../../tests/invariants/body_formation_work.rs"]
mod tests;

pub(crate) struct Formation<'a> {
    types: &'a Store,
    bounds: &'a Bounds,
    callables: &'a Templates,
    scan: Scan,
    slots: Slots,
    demand: Deferred,
}
impl<'a> Formation<'a> {
    pub(super) fn new(types: &'a Store, bounds: &'a Bounds, callables: &'a Templates) -> Self {
        Self {
            types,
            bounds,
            callables,
            scan: Scan::default(),
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

/// Distinct targets and policies must not reuse one another's component visits.
/// Each cursor retains graph nodes, never flattened results for every prefix.
#[derive(Default)]
struct Deferred {
    walks: HashMap<(RegionId, ComponentKind), Walk>,
}
impl Demand for Deferred {
    fn require(
        &mut self,
        scan: &mut Scan,
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
        let (regions, parameters, projections) = self
            .walks
            .entry((shorter, kind))
            .or_default()
            .extend(types, &[ty], depth, kind, &mut scan.budget)?;
        scan.budget
            .work(regions.len() + parameters.len() + projections.len())?;
        for longer in regions {
            scan.record((longer, shorter));
        }
        for parameter in parameters {
            let pair = (parameter, shorter);
            if scan.unique_parameters.insert(pair) {
                scan.parameter_pairs.push(pair);
            }
        }
        for projection in projections {
            let pair = (projection, shorter);
            if scan.unique_projections.insert(pair) {
                scan.projection_pairs.push(pair);
            }
        }
        Ok(())
    }
    fn projection(
        &mut self,
        scan: &mut Scan,
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
