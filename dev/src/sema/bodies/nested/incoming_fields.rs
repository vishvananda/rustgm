//! Only retained capture fields can supply an incoming conditional proof port.
use crate::support::hash::HashSet;
use crate::types::{RegionId, Store, TypeId, substitute::Bound};

#[derive(Default)]
pub(super) struct Inputs {
    regions: Option<HashSet<RegionId>>,
    pub steps: usize,
}
impl Inputs {
    pub(super) fn collect(types: &mut Store, fields: &[TypeId]) -> Result<Self, &'static str> {
        let mut this = Self::default();
        let mut observer = None::<Bound>;
        for &field in fields {
            this.charge(1)?;
            if !types
                .needs_region_context(field)
                .ok_or("missing capture field lifetime properties")?
            {
                continue;
            }
            let observer = observer.get_or_insert_with(Bound::observe_regions);
            let before = observer.steps;
            observer.apply(types, field)?;
            this.charge(observer.steps.saturating_sub(before))?;
        }
        if let Some(observer) = observer {
            for region in observer.observed_regions() {
                this.charge(1)?;
                if !matches!(region.bound_parts(), Some((0, _))) {
                    continue;
                }
                let regions = this.regions.get_or_insert_with(HashSet::default);
                if regions.len() >= 65_536 {
                    return Err("capture input lifetime index exceeds source binder budget");
                }
                regions.insert(region);
            }
        }
        Ok(this)
    }
    pub(super) fn contains(&self, region: RegionId) -> bool {
        self.regions
            .as_ref()
            .is_some_and(|regions| regions.contains(&region))
    }
    pub(super) fn is_empty(&self) -> bool {
        self.regions.as_ref().is_none_or(HashSet::is_empty)
    }
    fn charge(&mut self, amount: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(amount);
        if self.steps > 1_000_000 {
            return Err("capture input lifetime observation work budget exceeded");
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "../../../../tests/invariants/incoming_cells.rs"]
mod tests;

impl super::Checker<'_> {
    pub(super) fn capture_input_ports(
        &mut self,
        source: crate::resolve::unit::NodeKey,
        enabled: bool,
    ) -> Result<Inputs, super::Error> {
        if !enabled {
            return Ok(Inputs::default());
        }
        let Some(closure) = self
            .facts
            .closures
            .get(source)
            .filter(|closure| closure.execution.is_some())
        else {
            return Ok(Inputs::default());
        };
        let Some(environment) = self.facts.closures.environment(closure.identity) else {
            return Ok(Inputs::default());
        };
        let inputs = Inputs::collect(&mut self.facts.types, &environment.field_types)
            .map_err(|e| self.error(source.node, e))?;
        self.facts.body_work.canonical_steps += inputs.steps;
        Ok(inputs)
    }
    pub(super) fn validate_capture_input_parent(
        &self,
        source: crate::resolve::unit::NodeKey,
        region: RegionId,
        parent: crate::resolve::unit::NodeKey,
    ) -> Result<(), super::Error> {
        if !self
            .facts
            .body_regions
            .get(parent, region)
            .is_some_and(|fact| fact.body == parent)
        {
            return Err(self.error(
                source.node,
                "capture input bridge has a different parent lifetime owner",
            ));
        }
        Ok(())
    }
}
