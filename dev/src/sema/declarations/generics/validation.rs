//! Definition-time checks use no assumed proof for closed, non-generic bounds.
use super::{Checker, DefId, Error};
use crate::{
    sema::nominals::Proof,
    types::parameters::{
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
};
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
            let body = match predicate.value {
                Clause::Predicate(value) => value,
                Clause::ForAll(id) => {
                    self.facts
                        .types
                        .quantified(id)
                        .ok_or_else(|| self.error(predicate.source, "invalid declaration clause"))?
                        .predicate
                }
            };
            if matches!(body.bound, Bound::Trait(_)) {
                self.validate_trait_bound(owner, predicate, proof, &mut traits)?;
                continue;
            }
            if matches!(body.bound, Bound::ProjectionEqual(_)) {
                self.validate_equality_bound(predicate, proof)?;
                continue;
            }
            if matches!(body.bound, Bound::Outlives(_)) {
                if !proof
                    .form_clause_term(&mut self.facts, predicate.value)
                    .map_err(|e| self.error(predicate.source, e))?
                    .well_formed
                {
                    return Err(self.error(predicate.source, "bound subject must be well-formed"));
                }
                continue;
            }
            let original = self
                .facts
                .unreduced
                .predicate(predicate.source)
                .unwrap_or(predicate.value);
            let formation = proof
                .validate_builtin_clause(&mut self.facts, predicate.value, original)
                .map_err(|e| self.error(predicate.source, e))?;
            if !formation.satisfied {
                return Err(self.error(
                    predicate.source,
                    "non-generic bound is not satisfied at declaration",
                ));
            }
            if let Clause::ForAll(id) = predicate.value {
                self.facts
                    .generics
                    .record_builtin_formation(owner, id, formation.subject)
                    .map_err(|e| self.error(predicate.source, e))?;
            }
        }
        Ok(())
    }

    fn validate_trait_bound(
        &mut self,
        owner: DefId,
        predicate: crate::sema::generics::Predicate,
        proof: &mut Proof,
        state: &mut Option<TraitEvidence>,
    ) -> Result<(), Error> {
        let formation = proof
            .form_trait(&mut self.facts, predicate.value)
            .map_err(|e| self.error(predicate.source, e))?;
        if let Some(requirement) = formation.declaration() {
            if state.is_none() {
                *state = Some(self.declared_trait_assumptions(owner)?);
            }
            self.check_trait_assumption(
                state.as_mut().ok_or_else(|| {
                    self.error(predicate.source, "missing trait proof environment")
                })?,
                predicate,
                requirement,
                proof,
            )?;
        }
        let original = self
            .facts
            .unreduced
            .predicate(predicate.source)
            .unwrap_or(predicate.value);
        if original
            .requires_definition_proof(&self.facts.types)
            .map_err(|e| self.error(predicate.source, e))?
        {
            let requirement = formation.declaration().ok_or_else(|| {
                self.error(
                    predicate.source,
                    "closed trait clause has a local formation input",
                )
            })?;
            if !proof
                .prove_closed_trait(&mut self.facts, requirement)
                .map_err(|e| self.error(predicate.source, e))?
            {
                return Err(self.error(
                    predicate.source,
                    "closed trait bounds require an implementation proof",
                ));
            }
        }
        Ok(())
    }

    fn validate_equality_bound(
        &mut self,
        predicate: crate::sema::generics::Predicate,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        // Formation of the projected subject follows its separately retained trait
        // premise. An unused generic equality does not require its result to satisfy
        // the associated declaration's Sized/Copy promises.
        if !proof
            .form_clause_term(&mut self.facts, predicate.value)
            .map_err(|e| self.error(predicate.source, e))?
            .well_formed
        {
            return Err(self.error(
                predicate.source,
                "associated equality result must be well-formed",
            ));
        }
        let mut original = self
            .facts
            .unreduced
            .predicate(predicate.source)
            .unwrap_or(predicate.value);
        let original_body = match original {
            Clause::Predicate(value) => value,
            Clause::ForAll(id) => {
                self.facts
                    .types
                    .quantified(id)
                    .ok_or_else(|| self.error(predicate.source, "invalid equality clause"))?
                    .predicate
            }
        };
        if !matches!(original_body.bound, Bound::ProjectionEqual(_)) {
            original = predicate.value;
        }
        if !original
            .requires_definition_proof(&self.facts.types)
            .map_err(|e| self.error(predicate.source, e))?
        {
            return Ok(());
        }
        let original = match original {
            Clause::Predicate(value) => value,
            Clause::ForAll(id) => {
                let value = self
                    .facts
                    .types
                    .quantified(id)
                    .ok_or_else(|| self.error(predicate.source, "invalid equality clause"))?;
                crate::types::substitute::Bound::lower(1)
                    .predicate(&mut self.facts.types, value.predicate)
                    .map_err(|e| self.error(predicate.source, e))?
            }
        };
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
        requirement: Predicate,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        if state.assumptions.has_quantified() {
            let first = proof
                .region_frontier(&mut self.facts)
                .map_err(|e| self.error(predicate.source, e))?;
            if !state
                .assumptions
                .unambiguous_scoped_declaration(
                    &mut self.facts.types,
                    &mut state.context,
                    predicate.value,
                    requirement,
                    first,
                )
                .map_err(|e| self.error(predicate.source, e.to_string()))?
            {
                return Err(self.error(
                    predicate.source,
                    "ambiguous trait assumptions in declaration",
                ));
            }
            return Ok(());
        }
        if !state
            .assumptions
            .unambiguous_declaration(&mut self.facts.types, &mut state.context, requirement)
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
