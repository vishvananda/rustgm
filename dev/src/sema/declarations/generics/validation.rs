//! Definition-time checks use no assumed proof for closed, non-generic bounds.
use super::{Checker, DefId, Error};
use crate::types::parameters::{predicates::Bound, shape::Shapes};
impl Checker<'_> {
    pub(in super::super) fn validate_bounds(
        &mut self,
        owner: DefId,
        shapes: &mut Option<Shapes>,
    ) -> Result<(), Error> {
        for predicate in self
            .facts
            .generics
            .predicates
            .get(&owner)
            .into_iter()
            .flatten()
        {
            let ty = predicate.value.ty;
            let well_formed = if let Some(shapes) = shapes {
                shapes
                    .demand(&self.facts.types, ty)
                    .map_err(|e| self.error(predicate.source, e))?
                    .well_formed
            } else {
                self.facts.types.is_well_formed(ty) == Some(true)
            };
            if !well_formed {
                return Err(self.error(predicate.source, "bound subject must be well-formed"));
            }
            if matches!(predicate.value.bound, Bound::Outlives(_)) {
                continue;
            }
            if self.facts.types.contains_parameters(ty) != Some(false)
                || self.facts.types.escaping_binders(ty) != Some(0)
            {
                continue;
            }
            let valid = if predicate.value.bound == Bound::Sized {
                self.facts.types.is_sized(ty) == Some(true)
            } else {
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
