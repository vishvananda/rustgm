//! Exact exposure queries bind stable arenas; legacy dense queries keep their contract.
use super::{Definitions, HashSet, NeedsDrop, Scope, Store, TypeId};
use crate::{support::retained, types::arena::Identity};

pub(super) struct Binding {
    types: Identity,
    definitions: Identity,
}
impl Binding {
    pub(super) const fn allocation_bytes() -> usize {
        // Two arena tokens and the immutable custom-registry Arc control block.
        size_of::<Self>() + 3 * Identity::allocation_bytes()
    }
}
impl NeedsDrop {
    /// Fixed registry/header/token preflight precedes all new exact-family allocations.
    pub(crate) fn for_exposure(
        types: &mut Store,
        definitions: &mut Definitions,
        limit: usize,
    ) -> Result<Self, &'static str> {
        let fixed = retained::sum([
            size_of::<Self>(),
            size_of::<Scope>(),
            size_of::<HashSet<crate::types::ItemId>>(),
            Binding::allocation_bytes(),
        ]);
        if fixed > limit {
            return Err("exact exposure query exceeds its storage budget");
        }
        let mut query =
            Self::concrete_bounded(std::iter::empty(), limit - Binding::allocation_bytes())?;
        let scope = query
            .scope
            .as_deref_mut()
            .ok_or("missing exact exposure scope")?;
        scope.binding = Some(Box::new(Binding {
            types: types.arena_identity(),
            definitions: definitions.arena_identity(),
        }));
        Ok(query)
    }
    pub(crate) fn require_catalogue_environment(
        &self,
        types: &Store,
        definitions: &Definitions,
    ) -> Result<(), &'static str> {
        if let Some(binding) = self.scope.as_ref().and_then(|s| s.binding.as_deref())
            && (!types.has_identity(&binding.types)
                || !definitions.has_identity(&binding.definitions))
        {
            return Err("exact exposure query belongs to another arena");
        }
        Ok(())
    }
    /// Only the new bound exact mode reports owned query storage to the recipe budget.
    pub(crate) fn catalogue_bytes(&self) -> Result<Option<usize>, &'static str> {
        match &self.scope {
            None => Ok(None),
            Some(scope) if scope.environment.is_none() && scope.binding.is_some() => {
                Ok(Some(self.retained_bytes()))
            }
            Some(_) => Err("destruction catalog cannot consume an unbound or symbolic query"),
        }
    }
    pub(crate) fn demand_catalogue(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        root: TypeId,
        limit: usize,
    ) -> Result<bool, &'static str> {
        self.require_catalogue_environment(types, definitions)?;
        if self.catalogue_bytes()?.is_some() {
            self.demand_concrete_bounded(types, definitions, root, limit)
        } else {
            self.demand(types, definitions, root)
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/arena_queries.rs"]
mod tests;
