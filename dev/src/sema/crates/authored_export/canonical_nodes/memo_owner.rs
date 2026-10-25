//! Only actual anonymous type demand prepares this cold canonical arena anchor.
use super::{Declarations, Graph, Stage, TypeId};
use crate::types::{Store, arena::Identity};

impl Graph {
    pub(super) fn require_arena(&self, types: &Store) -> Result<(), &'static str> {
        if self
            .arena
            .as_ref()
            .is_some_and(|arena| !types.has_identity(arena))
        {
            return Err("source type memo belongs to another canonical arena");
        }
        Ok(())
    }
}
impl Stage {
    pub(in super::super) fn prepare_signature_arena(
        &mut self,
        facts: &mut Declarations,
        signature: TypeId,
    ) -> Result<(), &'static str> {
        let parts = super::signatures::Parts::actual(facts, signature)?;
        if let Some(ty) = self.anonymous_signature_field(facts, parts)? {
            self.prepare_type_arena(facts, ty)?;
        }
        Ok(())
    }

    pub(super) fn anonymous_signature_field(
        &mut self,
        facts: &Declarations,
        parts: super::signatures::Parts,
    ) -> Result<Option<TypeId>, &'static str> {
        let parameters = facts
            .types
            .list(parts.parameters)
            .ok_or("source callable input list is unknown")?;
        self.charge(parameters.len().saturating_add(1))?;
        Ok(parameters
            .iter()
            .copied()
            .chain(std::iter::once(parts.result))
            .find(|&ty| facts.types.contains_anonymous_callables(ty) == Some(true)))
    }

    pub(crate) fn prepare_type_arena(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<(), &'static str> {
        self.graph.require_arena(&facts.types)?;
        let anonymous = facts
            .types
            .contains_anonymous_callables(ty)
            .ok_or("source type root has no actual callable summary")?;
        if !anonymous {
            return Ok(());
        }
        self.charge(1)?;
        if facts.types.existing_arena_identity().is_none() {
            self.reserve(Identity::allocation_bytes())?;
            facts.types.arena_identity();
        }
        Ok(())
    }

    pub(super) fn bind_type_arena(&mut self, facts: &Declarations) -> Result<(), &'static str> {
        self.graph.require_arena(&facts.types)?;
        if self.graph.arena.is_none() {
            self.charge(1)?;
            self.graph.arena = Some(
                facts
                    .types
                    .existing_arena_identity()
                    .ok_or("source anonymous callable requires its actual prepared type arena")?
                    .clone(),
            );
        }
        Ok(())
    }
}
