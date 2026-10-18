//! Directed lifetime evidence retains private intervals and quantified binder ownership.
mod graft;
mod placement;
mod quantified;
mod roots;
mod scopes;
mod stream;
mod substitution;
use crate::types::{
    RegionId, TypeId,
    outlives::{Pair, Queries},
    relations::{Constraints, ProofScope},
};
pub use quantified::ScopeId;
pub(crate) use roots::TypeRoot;
pub(crate) use scopes::ScopePlan;
pub(crate) use scopes::map_scopes;
use std::ops::Range;

#[derive(Clone)]
pub struct Obligations {
    /// Allocation envelope. Streamed body evidence can exclude foreign intervals.
    pub regions: Range<u32>,
    pub equalities: Vec<Pair>,
    pub outlives: Vec<Pair>,
    pub types: Vec<(TypeId, RegionId)>,
    quantified: Option<Box<quantified::Quantified>>,
}
impl Obligations {
    #[must_use]
    pub fn new(first: u32) -> Self {
        Self {
            regions: first..first,
            equalities: Vec::new(),
            outlives: Vec::new(),
            types: Vec::new(),
            quantified: None,
        }
    }
    #[must_use]
    pub fn has_quantified(&self) -> bool {
        self.quantified.is_some()
    }
    /// Retained lexical scopes, excluding the caller's root scope.
    #[must_use]
    pub fn scope_count(&self) -> usize {
        self.quantified.as_ref().map_or(0, |q| q.scopes.len())
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.owned_regions().next().is_none()
            && self.equalities.is_empty()
            && self.outlives.is_empty()
            && self.types.is_empty()
            && self.quantified.as_ref().is_none_or(|q| q.is_empty())
    }
    /// Combine independently allocated evidence in its original proof order.
    /// # Errors
    /// Rejects noncontiguous intervals and exhausted quantified scope identities.
    pub fn extend(&mut self, mut proof: Self) -> Result<(), &'static str> {
        if proof.regions.start != self.regions.end || proof.regions.end < proof.regions.start {
            return Err("declaration trait proof region intervals disagree");
        }
        if let Some(quantified) = proof.quantified.take() {
            if let Some(existing) = &mut self.quantified {
                existing.extend(*quantified)?;
            } else {
                self.quantified = Some(quantified);
            }
        }
        self.regions.end = proof.regions.end;
        self.equalities.extend(proof.equalities);
        self.outlives.extend(proof.outlives);
        self.types.extend(proof.types);
        Ok(())
    }
    /// # Errors
    /// Rejects invalid variables/types and exhausted lifetime proof resources.
    pub fn require(
        &self,
        constraints: &mut Constraints<'_>,
        queries: &mut Queries<'_>,
    ) -> Result<(), String> {
        self.require_source(constraints, queries, 0)
    }
    /// Retain the caller's source identity through all nested proof scopes.
    /// # Errors
    /// Rejects malformed scopes, intervals and failed lifetime relations.
    pub fn require_source(
        &self,
        constraints: &mut Constraints<'_>,
        queries: &mut Queries<'_>,
        source: u32,
    ) -> Result<(), String> {
        if let Some(quantified) = &self.quantified {
            quantified.require(self, constraints, queries, source)?;
        } else {
            for id in self.owned_regions().flatten() {
                constraints
                    .variable(RegionId(id))
                    .map_err(|e| e.to_string())?;
            }
        }
        self.require_at(constraints, queries, ProofScope::ROOT, source)
    }
    fn require_at(
        &self,
        constraints: &mut Constraints<'_>,
        queries: &mut Queries<'_>,
        scope: ProofScope,
        source: u32,
    ) -> Result<(), String> {
        for &(a, b) in &self.equalities {
            constraints
                .scoped_outlives(a, b, scope, source)
                .map_err(|e| e.to_string())?;
            constraints
                .scoped_outlives(b, a, scope, source)
                .map_err(|e| e.to_string())?;
        }
        for &(a, b) in &self.outlives {
            constraints
                .scoped_outlives(a, b, scope, source)
                .map_err(|e| e.to_string())?;
        }
        for &(ty, shorter) in &self.types {
            constraints.require_type(queries, ty, shorter, scope, source)?;
        }
        Ok(())
    }
}
