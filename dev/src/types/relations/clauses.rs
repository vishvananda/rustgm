//! Assumptions instantiate a caller premise; requirements introduce fresh universal scopes.
use super::{Constraints, Error, Kind, ProofScope, RegionId};
use crate::types::{
    outlives::Queries,
    parameters::{
        clauses::{Clause, QuantifiedId},
        predicates::Bound,
    },
};

impl Constraints<'_> {
    /// Consume the outlives part of a complete caller clause. Other bound kinds
    /// remain the responsibility of their own formation and satisfaction checks.
    /// This never introduces a requirement's local scope as a new assumption.
    /// # Errors
    /// Rejects unknown identities, escaping captures and exhausted proof work.
    pub fn assume_outlives_clause(
        &mut self,
        queries: &mut Queries<'_>,
        clause: Clause,
    ) -> Result<(), String> {
        let id = match clause {
            Clause::Predicate(predicate) => {
                if let Bound::Outlives(shorter) = predicate.bound {
                    self.assume_type_outlives(queries, predicate.ty, shorter)?;
                }
                return Ok(());
            }
            Clause::ForAll(id) => id,
        };
        let value = self
            .types
            .quantified(id)
            .ok_or("invalid outlives assumption clause")?;
        let Bound::Outlives(shorter) = value.predicate.bound else {
            return Ok(());
        };
        let subject = self
            .types
            .kind(value.predicate.ty)
            .ok_or("invalid outlives assumption subject")?;
        if matches!(subject, Kind::Projection(_)) {
            return self
                .assume_quantified_projection(id)
                .map_err(|e| e.to_string());
        }
        let shorter = match shorter.bound_parts() {
            Some((0, _)) => {
                // Only the original parameter subject establishes an all-regions
                // bound. A composite with a local target contributes no components.
                if let Kind::Parameter(id) = subject {
                    self.assume_parameter(id, RegionId::STATIC)
                        .map_err(|e| e.to_string())?;
                }
                return Ok(());
            }
            Some((depth, slot)) => RegionId::bound(depth - 1, slot)?,
            None => shorter,
        };
        let parts = queries.clause_components(value.predicate.ty)?;
        for &longer in parts.scalars.regions {
            self.assume(longer, shorter).map_err(|e| e.to_string())?;
        }
        for &parameter in parts.scalars.parameters {
            self.assume_parameter(parameter, shorter)
                .map_err(|e| e.to_string())?;
        }
        for &projection in parts.projections {
            self.assume_projection(projection, shorter)?;
        }
        Ok(())
    }
    fn assume_quantified_projection(&mut self, id: QuantifiedId) -> Result<(), Error> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.limit {
            return Err(Error::Budget);
        }
        let patterns = self
            .projected
            .get_or_insert_with(Default::default)
            .quantified
            .get_or_insert_with(Default::default);
        let before = patterns.steps();
        let result = patterns.assume(self.types, id);
        self.steps = self
            .steps
            .saturating_add(patterns.steps().saturating_sub(before));
        if self.steps > self.limit {
            return Err(Error::Budget);
        }
        result
    }
    /// Require a complete outlives clause without promoting any of it to premises.
    /// Empty binders still preserve the coordinate shift of enclosing captures.
    /// Other clause kinds are checked by their separate semantic consumers.
    /// # Errors
    /// Rejects unknown clauses, foreign identities and exhausted scoped proof work.
    pub fn require_outlives_clause(
        &mut self,
        queries: &mut Queries<'_>,
        clause: Clause,
        source: u32,
    ) -> Result<(), String> {
        let (predicate, count) = match clause {
            Clause::Predicate(predicate) => (predicate, None),
            Clause::ForAll(id) => {
                let value = self
                    .types
                    .quantified(id)
                    .ok_or("invalid required outlives clause")?;
                (value.predicate, Some(value.bound_regions))
            }
        };
        let Bound::Outlives(shorter) = predicate.bound else {
            return Ok(());
        };
        let scope = if let Some(count) = count {
            self.universal_scope(ProofScope::ROOT, count)
                .map_err(|e| e.to_string())?
        } else {
            ProofScope::ROOT
        };
        self.require_type(queries, predicate.ty, shorter, scope, source)
    }
}
