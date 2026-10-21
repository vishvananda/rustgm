//! Each closure proves its body against fresh input universals, never caller locals.
use super::{Bound, Error};
use crate::{sema::bodies::calls::CallTypes, types::RegionId};
use std::collections::HashMap;

pub(super) struct Interface {
    pub opening: Bound,
    annotations: Vec<RegionId>,
    inherited: usize,
    contextual: Option<u32>,
}
impl Interface {
    pub fn new(
        enclosing: &[RegionId],
        mut annotations: Vec<RegionId>,
        contextual: Option<u32>,
    ) -> Result<Self, &'static str> {
        let mut opening = Bound::universals(0);
        for &region in enclosing {
            opening.region(region, 0)?;
        }
        let inputs = contextual.map_or(annotations.len(), |count| count as usize);
        if contextual.is_some() {
            // Elided annotations refine the contextual universals; they do not
            // introduce independent rigid lifetimes beside the expected binder.
            annotations.clear();
        }
        for index in 0..inputs {
            opening.region(slot(index)?, 0)?;
        }
        Ok(Self {
            opening,
            annotations,
            inherited: enclosing.len(),
            contextual,
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
            if !output
                || source.bound_parts().is_none()
                || source.bound_parts().is_some_and(|(depth, slot)| {
                    depth == 0 && self.contextual.is_some_and(|count| u32::from(slot) < count)
                })
            {
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
    /// Body contracts bind their own universals before any local variables exist.
    /// Keep the equality obligations, but restore signature aliases to those slots.
    pub(super) fn closure_contract_signature(
        &mut self,
        node: super::NodeId,
        signature: super::TypeId,
        solved: &crate::types::infer::Solved,
    ) -> Result<super::TypeId, Error> {
        if self.facts.types.needs_region_context(signature) == Some(false) {
            return Ok(signature);
        }
        let mut equations = crate::types::substitute::RegionEquations::default();
        let work = equations
            .extend(solved.region_equalities.iter().copied())
            .map_err(|e| self.error(node, e))?;
        // Only static and this body's bound slots may anchor a formal signature.
        let mut fold = Bound::close_projection_regions(1, 0, HashMap::new())
            .map_err(|e| self.error(node, e))?;
        fold.exchange_projection_equations(&mut equations)
            .map_err(|e| self.error(node, e))?;
        let signature = fold
            .signature(&mut self.facts.types, signature, self.state.body_binders)
            .map_err(|e| self.error(node, e))?;
        self.facts.body_work.canonical_steps += work + fold.steps;
        Ok(signature)
    }

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
