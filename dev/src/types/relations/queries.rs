//! Batch declaration queries reuse the proof vocabulary without requiring their answers.
use super::{Constraints, Error, Scope, identity_guard::View};
#[cfg(test)]
#[path = "../../../tests/invariants/input_contracts/answers.rs"]
mod tests;
use crate::types::{
    RegionId, TypeId,
    outlives::Queries,
    region_graph::{Answers, VerificationId},
};

impl Constraints<'_> {
    pub(crate) fn test_region(
        &mut self,
        longer: RegionId,
        shorter: RegionId,
    ) -> Result<VerificationId, Error> {
        self.graph.verify_outlives(
            self.region(longer, Scope(0))?,
            self.region(shorter, Scope(0))?,
        )
    }

    pub(crate) fn test_type(
        &mut self,
        queries: &mut Queries<'_>,
        ty: TypeId,
        shorter: RegionId,
    ) -> Result<VerificationId, Error> {
        let shorter = self.region(shorter, Scope(0))?;
        let parts = queries.components(ty).map_err(|_| Error::Identity)?;
        self.steps = self.steps.saturating_add(parts.len());
        if self.steps > self.limit {
            return Err(Error::Budget);
        }
        let mut conditions = Vec::with_capacity(parts.len());
        for &longer in parts.scalars.regions {
            conditions.push(
                self.graph
                    .verify_outlives(self.region(longer, Scope(0))?, shorter)?,
            );
        }
        for &id in parts.scalars.parameters {
            conditions.push(
                self.graph
                    .verify_outlives(*self.parameters.get(&id).ok_or(Error::Identity)?, shorter)?,
            );
        }
        for &projection in parts.projections {
            let mut state = self.projected.take().unwrap_or_default();
            let result = state.test(
                self,
                View {
                    projection,
                    scope: Scope(0),
                },
                shorter,
            );
            self.projected = Some(state);
            conditions.push(result?);
        }
        self.graph.verify_all(&conditions)
    }

    pub(crate) fn test_answers(self, roots: &[VerificationId]) -> Result<Answers, Error> {
        if self
            .projected
            .as_ref()
            .is_some_and(|s| !s.pending.is_empty())
        {
            return Err(Error::Identity);
        }
        let mut result = self.graph.test_verifications(roots)?;
        result.steps = result.steps.saturating_add(self.steps);
        if result.steps > self.limit {
            return Err(Error::Budget);
        }
        Ok(result)
    }
}
