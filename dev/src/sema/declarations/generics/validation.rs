//! Definition-time checks use no assumed proof for closed, non-generic bounds.
use super::{Checker, DefId, Error};
use crate::types::Kind;
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
            self.bound_region_boundary(predicate.source, ty)?;
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
    fn bound_region_boundary(
        &self,
        key: crate::resolve::unit::NodeKey,
        ty: crate::types::TypeId,
    ) -> Result<(), Error> {
        if self.facts.types.contains_parameters(ty) != Some(true)
            || self.facts.types.contains_regions(ty) != Some(true)
        {
            return Ok(());
        }
        let mut work = vec![(ty, 0_u16)];
        let mut seen = std::collections::HashSet::new();
        let mut uses = crate::types::parameters::uses::Uses::default();
        while let Some((ty, depth)) = work.pop() {
            if !seen.insert((ty, depth)) {
                continue;
            }
            if seen.len() + work.len() > 1_000_000 {
                return Err(self.error(key, "bound well-formedness work budget exceeded"));
            }
            let list = match self
                .facts
                .types
                .kind(ty)
                .ok_or_else(|| self.error(key, "invalid bound subject"))?
            {
                Kind::Reference {
                    referent, region, ..
                } => {
                    if region
                        .bound_parts()
                        .is_none_or(|(binder, _)| binder >= depth)
                    {
                        uses.collect(&self.facts.types, referent)
                            .map_err(|e| self.error(key, e))?;
                        if !uses.parameters.is_empty() {
                            return Err(self.error(
                                key,
                                "bound well-formedness requires symbolic type-outlives proofs",
                            ));
                        }
                    }
                    work.push((referent, depth));
                    None
                }
                Kind::Array { element, .. }
                | Kind::ArrayTerm { element, .. }
                | Kind::Slice(element)
                | Kind::Pointer {
                    pointee: element, ..
                } => {
                    work.push((element, depth));
                    None
                }
                Kind::Tuple(list) => Some((list, depth)),
                Kind::Function {
                    parameters, result, ..
                } => {
                    let depth = depth
                        .checked_add(1)
                        .ok_or_else(|| self.error(key, "bound binder depth overflow"))?;
                    work.push((result, depth));
                    Some((parameters, depth))
                }
                _ => None,
            };
            if let Some((list, depth)) = list {
                let types = self
                    .facts
                    .types
                    .list(list)
                    .ok_or_else(|| self.error(key, "invalid bound type list"))?;
                if seen
                    .len()
                    .saturating_add(work.len())
                    .saturating_add(types.len())
                    > 1_000_000
                {
                    return Err(self.error(key, "bound well-formedness work budget exceeded"));
                }
                work.extend(types.iter().map(|ty| (*ty, depth)));
            }
        }
        Ok(())
    }
}
