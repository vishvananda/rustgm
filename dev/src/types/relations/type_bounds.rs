//! Typed well-formedness and outlives clauses share one scoped relation graph.
use super::{
    Constraints, Error, ProofScope, Scope, identity_guard::View, projected_bounds::Request,
};
use crate::types::{
    RegionId, TypeId,
    outlives::{BoundParts, Projection, Queries, Requirements},
};

impl Constraints<'_> {
    /// Assume exactly the structural requirements implied by this declared type.
    /// Opaque result clauses retain their complete projection identity.
    /// # Errors
    /// Rejects invalid region/type identities and exhausted proof storage.
    pub fn assume_requirements(&mut self, requirements: Requirements<'_>) -> Result<(), String> {
        for &(a, b) in requirements.scalars.regions {
            self.assume(a, b).map_err(|e| e.to_string())?;
        }
        for &(id, shorter) in requirements.scalars.parameters {
            self.assume_parameter(id, shorter)
                .map_err(|e| e.to_string())?;
        }
        for &(projection, shorter) in requirements.projections {
            self.assume_projection(projection, shorter)?;
        }
        Ok(())
    }
    pub(super) fn assume_projection(
        &mut self,
        projection: Projection,
        shorter: RegionId,
    ) -> Result<(), String> {
        let Some(identity) = self.types.result_identity(projection.ty) else {
            return Err("invalid outlives projection".into());
        };
        let longer = self.region(shorter, Scope(0)).map_err(|e| e.to_string())?;
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.limit {
            return Err(Error::Budget.to_string());
        }
        self.projected.get_or_insert_with(Default::default).assume(
            identity,
            View {
                projection,
                scope: Scope(0),
            },
            longer,
        );
        Ok(())
    }
    /// Require well-formedness without adding its clauses to the environment.
    /// # Errors
    /// Rejects invalid identities and exhausted scoped proof resources.
    pub fn require_requirements(
        &mut self,
        requirements: Requirements<'_>,
        scope: ProofScope,
        source: u32,
    ) -> Result<(), String> {
        for &(a, b) in requirements.scalars.regions {
            self.scoped_outlives(a, b, scope, source)
                .map_err(|e| e.to_string())?;
        }
        for &(id, shorter) in requirements.scalars.parameters {
            self.scoped_parameter_outlives(id, shorter, scope, source)
                .map_err(|e| e.to_string())?;
        }
        for &(projection, shorter) in requirements.projections {
            self.require_projection(projection, shorter, scope, source)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    /// Retain a projected type test until all ordinary constraints are known.
    /// This does not promote the result or its inputs to assumptions.
    /// # Errors
    /// Rejects foreign scopes/identities and exhausted proof resources.
    pub fn require_projection(
        &mut self,
        projection: Projection,
        shorter: RegionId,
        scope: ProofScope,
        source: u32,
    ) -> Result<(), Error> {
        let shorter = self.region(shorter, scope.scope)?;
        if self.types.result_identity(projection.ty).is_none() {
            return Err(Error::Identity);
        }
        self.steps = self.steps.saturating_add(1);
        if self.steps > self.limit {
            return Err(Error::Budget);
        }
        self.projected
            .get_or_insert_with(Default::default)
            .pending
            .push(Request {
                view: View {
                    projection,
                    scope: scope.scope,
                },
                shorter,
                source,
            });
        Ok(())
    }
    /// Require a type bound in its actual quantified scope.
    /// # Errors
    /// Rejects malformed types, foreign identities and exhausted proof resources.
    pub fn require_type(
        &mut self,
        queries: &mut Queries<'_>,
        ty: TypeId,
        shorter: RegionId,
        scope: ProofScope,
        source: u32,
    ) -> Result<(), String> {
        self.require_parts(queries.components(ty)?, shorter, scope, source)
    }
    fn require_parts(
        &mut self,
        parts: BoundParts<'_>,
        shorter: RegionId,
        scope: ProofScope,
        source: u32,
    ) -> Result<(), String> {
        for &longer in parts.scalars.regions {
            self.scoped_outlives(longer, shorter, scope, source)
                .map_err(|e| e.to_string())?;
        }
        for &id in parts.scalars.parameters {
            self.scoped_parameter_outlives(id, shorter, scope, source)
                .map_err(|e| e.to_string())?;
        }
        for &projection in parts.projections {
            self.require_projection(projection, shorter, scope, source)
                .map_err(|e| e.to_string())?;
        }
        Ok(())
    }
    pub(super) fn verify_projections(&mut self) -> Result<(), Error> {
        if let Some(mut state) = self.projected.take() {
            state.prove(self)?;
        }
        Ok(())
    }
}
