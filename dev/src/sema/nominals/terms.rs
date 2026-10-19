//! Scoped formation distinguishes recursive clause terms from whole builtin subjects.
use super::{Declarations, Proof};
use crate::types::{
    TypeId,
    parameters::{
        clauses::Clause,
        predicates::{Bound, Predicate},
        shape::Properties,
    },
};

pub(in crate::sema) struct BuiltinFormation {
    pub subject: Option<TypeId>,
    pub satisfied: bool,
}

impl Proof {
    /// Check equality results and outlives subjects inside their original binder.
    /// As with `check`, lifetime well-formedness still needs the final region gate.
    /// This does not prove the equality or outlives predicate itself.
    pub fn form_clause_term(
        &mut self,
        facts: &mut Declarations,
        clause: Clause,
    ) -> Result<Properties, &'static str> {
        let (value, count) = self.clause_context(facts, clause)?;
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

    /// Normalize with scoped evidence, then defer a whole subject containing locals.
    /// Closed builtin requirements must not justify themselves through assumptions.
    pub fn validate_builtin_clause(
        &mut self,
        facts: &mut Declarations,
        clause: Clause,
        original: Clause,
    ) -> Result<BuiltinFormation, &'static str> {
        let (value, count) = self.clause_context(facts, clause)?;
        if !matches!(value.bound, Bound::Copy | Bound::Clone | Bound::Sized) {
            return Err("builtin declaration checking requires a builtin bound");
        }
        let mut ty = if let Some(count) = count {
            self.normalize_clause_term(facts, value.ty, count)?
        } else {
            self.normalize(facts, value.ty)?
        };
        if count.is_some() {
            let Some(lifted) = self
                .builtin_inputs
                .get_or_insert_with(Default::default)
                .lift(&mut facts.types, ty)?
            else {
                return Ok(BuiltinFormation {
                    subject: None,
                    satisfied: true,
                });
            };
            ty = lifted;
        }
        if !self.check(facts, ty)?.well_formed {
            return Err("bound subject must be well-formed");
        }
        if !original.requires_definition_proof(&facts.types)? {
            return Ok(BuiltinFormation {
                subject: Some(ty),
                satisfied: true,
            });
        }
        let satisfied = Self::closed_builtin(facts, Predicate { ty, ..value })?;
        Ok(BuiltinFormation {
            subject: Some(ty),
            satisfied,
        })
    }

    fn closed_builtin(facts: &mut Declarations, value: Predicate) -> Result<bool, &'static str> {
        let ty = value.ty;
        if value.bound == Bound::Sized {
            return Ok(facts.types.is_sized(ty) == Some(true));
        }
        if facts.generics.has_nominal_requirements {
            facts.aggregates.prepare(&mut facts.types, ty)?;
        }
        let properties =
            facts
                .copies
                .demand(&facts.types, &facts.aggregates, &facts.derives, ty)?;
        Ok(if value.bound == Bound::Copy {
            properties.copy
        } else {
            properties.clone_
        })
    }

    fn clause_context(
        &self,
        facts: &Declarations,
        clause: Clause,
    ) -> Result<(Predicate, Option<u32>), &'static str> {
        match clause {
            Clause::Predicate(value) => Ok((value, None)),
            Clause::ForAll(id) => {
                let value = facts
                    .types
                    .quantified(id)
                    .ok_or("invalid formation clause")?;
                if self.scope.is_none() {
                    return Err("clause term formation requires an explicit declaration scope");
                }
                Ok((value.predicate, Some(value.bound_regions)))
            }
        }
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
                .builtin_inputs
                .as_ref()
                .map_or(0, |inputs| inputs.steps())
            + self
                .traits
                .as_ref()
                .map_or(0, |state| state.selection_steps())
    }
}

/// One proof owns the capture and lowering caches for all its builtin subjects.
pub(super) struct Inputs {
    captures: crate::types::bound_uses::captures::Captures,
    lower: crate::types::substitute::Bound,
    work: usize,
}
impl Default for Inputs {
    fn default() -> Self {
        Self {
            captures: crate::types::bound_uses::captures::Captures::new(1),
            lower: crate::types::substitute::Bound::lower(1),
            work: 0,
        }
    }
}
impl Inputs {
    fn steps(&self) -> usize {
        self.work
            .saturating_add(self.captures.steps)
            .saturating_add(self.lower.steps)
    }
    fn lift(
        &mut self,
        store: &mut crate::types::Store,
        ty: TypeId,
    ) -> Result<Option<TypeId>, &'static str> {
        self.work = self.work.saturating_add(1);
        if self.steps() > 1_000_000 {
            return Err("builtin formation work budget exceeded");
        }
        let value = if self.captures.collect(store, ty)? {
            None
        } else {
            Some(self.lower.apply(store, ty)?)
        };
        if self.steps() > 1_000_000 {
            return Err("builtin formation work budget exceeded");
        }
        Ok(value)
    }
}
