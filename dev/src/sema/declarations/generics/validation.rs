//! Definition-time checks use no assumed proof for closed, non-generic bounds.
use super::{Checker, DefId, Error};
use crate::{sema::nominals::Proof, types::parameters::predicates::Bound};
struct TraitEvidence {
    assumptions: crate::sema::traits::assumptions::Assumptions,
    context: crate::types::infer::Context,
}
impl Checker<'_> {
    pub(in super::super) fn validate_bounds(
        &mut self,
        owner: DefId,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        let mut traits = None;
        for index in 0..self
            .facts
            .generics
            .predicates
            .get(&owner)
            .map_or(0, Vec::len)
        {
            let predicate = self.facts.generics.predicates[&owner][index];
            let value = predicate
                .value
                .ordinary()
                .map_err(|error| self.error(predicate.source, error))?;
            if matches!(value.bound, Bound::ProjectionEqual(_)) {
                self.validate_equality_bound(predicate, proof)?;
                continue;
            }
            let ty = proof
                .normalize(&mut self.facts, value.ty)
                .map_err(|e| self.error(predicate.source, e))?;
            let well_formed = proof
                .check(&mut self.facts, ty)
                .map_err(|e| self.error(predicate.source, e))?
                .well_formed;
            if !well_formed {
                return Err(self.error(predicate.source, "bound subject must be well-formed"));
            }
            if matches!(value.bound, Bound::Trait(_)) {
                self.trait_requirements(predicate, proof)?;
                if traits.is_none() {
                    traits = Some(self.declared_trait_assumptions(owner)?);
                }
                self.check_trait_assumption(
                    traits.as_mut().ok_or_else(|| {
                        self.error(predicate.source, "missing trait proof environment")
                    })?,
                    predicate,
                )?;
            }
            if matches!(value.bound, Bound::Outlives(_)) {
                continue;
            }
            let original = self
                .facts
                .unreduced
                .predicate(predicate.source)
                .unwrap_or(predicate.value)
                .ordinary()
                .map_err(|error| self.error(predicate.source, error))?;
            if !original
                .requires_definition_proof(&self.facts.types)
                .map_err(|e| self.error(predicate.source, e))?
            {
                continue;
            }
            if matches!(value.bound, Bound::Trait(_)) {
                if !proof
                    .prove_closed_trait(&mut self.facts, value)
                    .map_err(|e| self.error(predicate.source, e))?
                {
                    return Err(self.error(
                        predicate.source,
                        "closed trait bounds require an implementation proof",
                    ));
                }
                continue;
            }
            let valid = self
                .closed_builtin_bound(ty, value.bound)
                .map_err(|e| self.error(predicate.source, e))?;
            if !valid {
                return Err(self.error(
                    predicate.source,
                    "non-generic bound is not satisfied at declaration",
                ));
            }
        }
        Ok(())
    }

    /// Closed builtin bounds have no caller assumptions to supply their abilities.
    fn closed_builtin_bound(
        &mut self,
        ty: crate::types::TypeId,
        bound: Bound,
    ) -> Result<bool, &'static str> {
        if bound == Bound::Sized {
            return Ok(self.facts.types.is_sized(ty) == Some(true));
        }
        if self.facts.generics.has_nominal_requirements {
            self.facts.aggregates.prepare(&mut self.facts.types, ty)?;
        }
        let properties = self.facts.copies.demand(
            &self.facts.types,
            &self.facts.aggregates,
            &self.facts.derives,
            ty,
        )?;
        Ok(if bound == Bound::Copy {
            properties.copy
        } else {
            properties.clone_
        })
    }

    fn validate_equality_bound(
        &mut self,
        predicate: crate::sema::generics::Predicate,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        let value = predicate
            .value
            .ordinary()
            .map_err(|error| self.error(predicate.source, error))?;
        let Bound::ProjectionEqual(result) = value.bound else {
            return Err(self.error(predicate.source, "missing associated equality result"));
        };
        // Formation of the projected subject follows its separately retained trait
        // premise. An unused generic equality does not require its result to satisfy
        // the associated declaration's Sized/Copy promises.
        if !proof
            .check(&mut self.facts, result)
            .map_err(|e| self.error(predicate.source, e))?
            .well_formed
        {
            return Err(self.error(
                predicate.source,
                "associated equality result must be well-formed",
            ));
        }
        let original = self
            .facts
            .unreduced
            .predicate(predicate.source)
            .map(crate::types::parameters::clauses::Clause::ordinary)
            .transpose()
            .map_err(|error| self.error(predicate.source, error))?
            .filter(|value| matches!(value.bound, Bound::ProjectionEqual(_)))
            .unwrap_or(value);
        if !original
            .requires_definition_proof(&self.facts.types)
            .map_err(|e| self.error(predicate.source, e))?
        {
            return Ok(());
        }
        if !proof
            .prove_closed_equality(&mut self.facts, original)
            .map_err(|e| self.error(predicate.source, e))?
        {
            return Err(self.error(
                predicate.source,
                "closed associated equality is not satisfied",
            ));
        }
        Ok(())
    }

    fn declared_trait_assumptions(&self, owner: DefId) -> Result<TraitEvidence, Error> {
        let mut assumptions = crate::sema::traits::assumptions::Assumptions::default();
        for predicate in self.facts.generics.predicates_in_scope(owner) {
            let value = match predicate.value {
                crate::types::parameters::clauses::Clause::Predicate(value) => value,
                crate::types::parameters::clauses::Clause::ForAll(id) => {
                    self.facts
                        .types
                        .quantified(id)
                        .ok_or_else(|| self.error(predicate.source, "invalid declaration clause"))?
                        .predicate
                }
            };
            if matches!(value.bound, Bound::Trait(_)) {
                assumptions
                    .insert_clause(&self.facts.types, predicate.value)
                    .map_err(|e| self.error(predicate.source, e.to_string()))?;
            }
        }
        Ok(TraitEvidence {
            assumptions,
            context: crate::types::infer::Context::default(),
        })
    }

    fn check_trait_assumption(
        &mut self,
        state: &mut TraitEvidence,
        predicate: crate::sema::generics::Predicate,
    ) -> Result<(), Error> {
        let value = predicate
            .value
            .ordinary()
            .map_err(|error| self.error(predicate.source, error))?;
        if !state
            .assumptions
            .unambiguous_declaration(&mut self.facts.types, &mut state.context, value)
            .map_err(|e| self.error(predicate.source, e.to_string()))?
        {
            return Err(self.error(
                predicate.source,
                "ambiguous trait assumptions in declaration",
            ));
        }
        Ok(())
    }
}
