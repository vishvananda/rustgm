//! Recursive clause-term formation defers individual local debts, not whole inputs.
use super::{Declarations, Proof};
use crate::types::{
    TypeId,
    parameters::{clauses::Clause, predicates::Bound, shape::Properties},
};

impl Proof {
    /// Check equality results and outlives subjects inside their original binder.
    /// As with `check`, lifetime well-formedness still needs the final region gate.
    /// This does not prove the equality or outlives predicate itself.
    pub fn form_clause_term(
        &mut self,
        facts: &mut Declarations,
        clause: Clause,
    ) -> Result<Properties, &'static str> {
        let (value, count) = match clause {
            Clause::Predicate(value) => (value, None),
            Clause::ForAll(id) => {
                let value = facts
                    .types
                    .quantified(id)
                    .ok_or("invalid formation clause")?;
                if self.scope.is_none() {
                    return Err("clause term formation requires an explicit declaration scope");
                }
                (value.predicate, Some(value.bound_regions))
            }
        };
        let ty = match value.bound {
            Bound::ProjectionEqual(result) => result,
            Bound::Outlives(_) => {
                if let Some(count) = count {
                    self.normalize_clause_term(facts, value.ty, count)?
                } else {
                    self.normalize(facts, value.ty)?
                }
            }
            _ => return Err("clause term formation requires an equality or outlives predicate"),
        };
        self.check_inside(facts, ty, count)
    }

    pub(super) fn normalize_clause_term(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
        count: u32,
    ) -> Result<TypeId, &'static str> {
        if facts.types.contains_projections(ty) != Some(true) {
            return Ok(ty);
        }
        let owner = self
            .scope
            .ok_or("clause normalization requires an explicit declaration scope")?;
        self.trait_state(facts)?
            .normalize_term(facts, ty, count, owner)
    }

    #[cfg(test)]
    pub(in crate::sema) fn formation_steps(&self) -> usize {
        self.checked.steps()
            + self
                .shapes
                .as_ref()
                .map_or(0, crate::types::parameters::shape::Shapes::steps)
            + self.term_shapes.as_ref().map_or(0, |shapes| shapes.steps())
            + self
                .traits
                .as_ref()
                .map_or(0, |state| state.selection_steps())
    }
}
