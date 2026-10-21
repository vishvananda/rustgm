//! Each closure proves its body against fresh input universals, never caller locals.
use super::{Bound, Error};
use crate::{sema::bodies::calls::CallTypes, types::RegionId};
use std::collections::HashMap;

pub(super) struct Interface {
    pub opening: Bound,
    annotations: Vec<RegionId>,
    inherited: usize,
}
impl Interface {
    pub fn new(enclosing: &[RegionId], annotations: Vec<RegionId>) -> Result<Self, &'static str> {
        let mut opening = Bound::universals(0);
        for &region in enclosing {
            opening.region(region, 0)?;
        }
        for index in 0..annotations.len() {
            opening.region(slot(index)?, 0)?;
        }
        Ok(Self {
            opening,
            annotations,
            inherited: enclosing.len(),
        })
    }
    pub fn count(&self) -> Result<u32, &'static str> {
        u32::try_from(self.opening.universal_count()).map_err(|_| "too many closure lifetimes")
    }
    pub fn equations(&self) -> Result<Vec<(RegionId, RegionId)>, &'static str> {
        self.annotations
            .iter()
            .enumerate()
            .map(|(index, &annotation)| Ok((annotation, slot(self.inherited + index)?)))
            .collect()
    }
    pub fn closing(&self, calls: &CallTypes, output: bool) -> Result<Bound, &'static str> {
        let mut mapping = HashMap::new();
        for (source, universal) in self.opening.universal_pairs() {
            if !output || source.bound_parts().is_none() {
                mapping.insert(universal, source);
            }
        }
        for (index, alias) in calls.universal_arguments().into_iter().enumerate() {
            if let Some(&target) = mapping.get(&slot(index)?) {
                mapping.insert(alias, target);
            }
        }
        if !output {
            for (index, &annotation) in self.annotations.iter().enumerate() {
                mapping.insert(annotation, slot(index)?);
            }
        }
        Ok(Bound::restore_regions(mapping))
    }
}
fn slot(index: usize) -> Result<RegionId, &'static str> {
    RegionId::bound(
        0,
        u16::try_from(index).map_err(|_| "too many closure lifetimes")?,
    )
}

impl super::Checker<'_> {
    pub(super) fn closure_result_boundary(&mut self, interface: &Interface) -> Result<(), Error> {
        let node = self.state.body.node;
        let result = self
            .state
            .result_term
            .ok_or_else(|| self.error(node, "missing closure output"))?;
        let mut closing = interface
            .closing(&self.state.call_types, true)
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .map_regions(&mut self.facts.types, result, &mut closing)
            .map_err(|_| {
                self.error(
                    node,
                    "closure result retains an input-only or local lifetime",
                )
            })?;
        Ok(())
    }
}
