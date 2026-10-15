//! A declaration proof owns a disjoint interval of fresh lifetimes and all its obligations.
use crate::types::{
    RegionId, TypeId,
    outlives::{Pair, Queries},
    relations::Constraints,
};
use std::ops::Range;

pub(crate) struct Obligations {
    pub regions: Range<u32>,
    pub equalities: Vec<Pair>,
    pub outlives: Vec<Pair>,
    pub types: Vec<(TypeId, RegionId)>,
}
impl Obligations {
    pub fn new(first: u32) -> Self {
        Self {
            regions: first..first,
            equalities: Vec::new(),
            outlives: Vec::new(),
            types: Vec::new(),
        }
    }
    pub(in crate::sema) fn append(
        &mut self,
        evidence: super::selection::Evidence,
    ) -> Result<(), &'static str> {
        if evidence.regions.start != self.regions.end {
            return Err("declaration trait proof region intervals disagree");
        }
        self.regions.end = evidence.regions.end;
        self.equalities.extend(evidence.equalities);
        self.outlives.extend(evidence.outlives);
        self.types.extend(evidence.type_outlives);
        Ok(())
    }
    pub fn require(
        &self,
        constraints: &mut Constraints<'_>,
        queries: &mut Queries<'_>,
    ) -> Result<(), String> {
        for id in self.regions.clone() {
            constraints
                .variable(RegionId(id))
                .map_err(|e| e.to_string())?;
        }
        for &(a, b) in &self.equalities {
            constraints.outlives(a, b, 0).map_err(|e| e.to_string())?;
            constraints.outlives(b, a, 0).map_err(|e| e.to_string())?;
        }
        for &(a, b) in &self.outlives {
            constraints.outlives(a, b, 0).map_err(|e| e.to_string())?;
        }
        for &(ty, shorter) in &self.types {
            let parts = queries.components(ty)?;
            for &longer in parts.regions {
                constraints
                    .outlives(longer, shorter, 0)
                    .map_err(|e| e.to_string())?;
            }
            for &id in parts.parameters {
                constraints
                    .parameter_outlives(id, shorter, 0)
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }
}
