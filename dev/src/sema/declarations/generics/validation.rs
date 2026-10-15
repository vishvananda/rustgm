//! Definition-time checks use no assumed proof for closed, non-generic bounds.
use super::{Checker, DefId, Error};
use crate::{sema::nominals::Proof, types::parameters::predicates::Bound};
impl Checker<'_> {
    pub(in super::super) fn validate_bounds(
        &mut self,
        owner: DefId,
        proof: &mut Proof,
    ) -> Result<(), Error> {
        for index in 0..self
            .facts
            .generics
            .predicates
            .get(&owner)
            .map_or(0, Vec::len)
        {
            let predicate = self.facts.generics.predicates[&owner][index];
            let ty = predicate.value.ty;
            let well_formed = proof
                .check(&mut self.facts, ty)
                .map_err(|e| self.error(predicate.source, e))?
                .well_formed;
            if !well_formed {
                return Err(self.error(predicate.source, "bound subject must be well-formed"));
            }
            if matches!(predicate.value.bound, Bound::Trait(_)) {
                self.trait_requirements(predicate, proof)?;
            }
            if matches!(predicate.value.bound, Bound::Outlives(_)) {
                continue;
            }
            if predicate
                .value
                .contains_parameters(&self.facts.types)
                .map_err(|e| self.error(predicate.source, e))?
                || predicate
                    .value
                    .escaping_binders(&self.facts.types)
                    .map_err(|e| self.error(predicate.source, e))?
                    != 0
            {
                continue;
            }
            if matches!(predicate.value.bound, Bound::Trait(_)) {
                return Err(self.error(
                    predicate.source,
                    "closed trait bounds require an implementation proof",
                ));
            }
            let valid = if predicate.value.bound == Bound::Sized {
                self.facts.types.is_sized(ty) == Some(true)
            } else {
                if self.facts.generics.has_nominal_requirements {
                    self.facts
                        .aggregates
                        .prepare(&mut self.facts.types, ty)
                        .map_err(|e| self.error(predicate.source, e))?;
                }
                let properties = self
                    .facts
                    .copies
                    .demand(
                        &self.facts.types,
                        &self.facts.aggregates,
                        &self.facts.derives,
                        ty,
                    )
                    .map_err(|e| self.error(predicate.source, e))?;
                if predicate.value.bound == Bound::Copy {
                    properties.copy
                } else {
                    properties.clone_
                }
            };
            if !valid {
                return Err(self.error(
                    predicate.source,
                    "non-generic bound is not satisfied at declaration",
                ));
            }
        }
        Ok(())
    }
}
