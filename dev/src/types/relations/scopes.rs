//! Quantified requirements introduce placeholders and private variables in their actual universe.
use super::{Constraints, Error, RegionId, RegionKind, Scope};
use crate::types::parameters::ParameterId;

/// A scope belongs to the constraint graph that created it. The root denotes
/// its caller's signature and inference context; nested scopes preserve captures.
#[derive(Clone, Copy)]
pub struct ProofScope {
    scope: Scope,
    universe: u32,
}
impl ProofScope {
    pub const ROOT: Self = Self {
        scope: Scope(0),
        universe: 0,
    };
}
impl Constraints<'_> {
    /// Add one universal binder without introducing assumptions about its locals.
    /// Empty binders preserve coordinates without creating a younger universe.
    /// # Errors
    /// Rejects missing parents, exhausted identities and graph work limits.
    pub fn universal_scope(&mut self, parent: ProofScope, count: u32) -> Result<ProofScope, Error> {
        self.binders.get(parent.scope.0).ok_or(Error::Identity)?;
        self.steps = self.steps.saturating_add(1);
        if self.steps >= self.limit {
            return Err(Error::Budget);
        }
        let universe = if count == 0 {
            parent.universe
        } else {
            self.universe = self.universe.checked_add(1).ok_or(Error::Budget)?;
            self.universe
        };
        let scope = self.binder(Some(parent.scope), count, true, universe)?;
        Ok(ProofScope { scope, universe })
    }

    /// Register a proof-private existential beneath the selected universal binder.
    /// A nested placeholder need not contain the caller's local storage extent.
    /// # Errors
    /// Rejects duplicate/non-free identities, foreign scopes and storage limits.
    pub fn scoped_variable(&mut self, id: RegionId, scope: ProofScope) -> Result<(), Error> {
        self.binders.get(scope.scope.0).ok_or(Error::Identity)?;
        if scope.universe == 0 {
            return self.variable(id);
        }
        if id == RegionId::STATIC || id.bound_parts().is_some() || self.variables.contains_key(&id)
        {
            return Err(Error::Identity);
        }
        let variable = self.graph.region(RegionKind::Variable {
            universe: scope.universe,
        })?;
        self.variables.insert(id, variable);
        Ok(())
    }

    /// Require a relation using bound lifetimes relative to this proof's scope.
    /// # Errors
    /// Rejects foreign lifetimes/scopes and exhausted graph resources.
    pub fn scoped_outlives(
        &mut self,
        longer: RegionId,
        shorter: RegionId,
        scope: ProofScope,
        source: u32,
    ) -> Result<(), Error> {
        self.graph.require(
            self.region(longer, scope.scope)?,
            self.region(shorter, scope.scope)?,
            source,
        )
    }

    /// Relate an outer type parameter to a lifetime in a quantified requirement.
    /// # Errors
    /// Rejects foreign parameters/lifetimes and exhausted graph resources.
    pub fn scoped_parameter_outlives(
        &mut self,
        parameter: ParameterId,
        shorter: RegionId,
        scope: ProofScope,
        source: u32,
    ) -> Result<(), Error> {
        self.graph.require(
            *self.parameters.get(&parameter).ok_or(Error::Identity)?,
            self.region(shorter, scope.scope)?,
            source,
        )
    }
}
