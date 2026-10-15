//! Body obligations share the same fixed assumptions as repeat and shape queries.
use super::{Checker, DefId, Error, NodeId, TypeId};
use crate::types::{
    Kind,
    infer::arguments::Argument,
    parameters::{ParameterId, predicates::Bound, shape::Properties},
    substitute,
};
impl Checker<'_> {
    pub(super) fn nominal_requirements(
        &mut self,
        node: NodeId,
        ty: TypeId,
        checked: &mut std::collections::HashSet<TypeId>,
    ) -> Result<(), Error> {
        if !self.facts.generics.has_nominal_requirements {
            return Ok(());
        }
        let requirements = crate::sema::nominals::requirements(self.facts, ty, checked)
            .map_err(|e| self.error(node, e))?;
        for requirement in requirements {
            self.prove_bound(node, requirement.ty, requirement.bound)?;
        }
        Ok(())
    }
    pub(super) fn prepare_bound_environment(
        &mut self,
        node: NodeId,
        owner: DefId,
    ) -> Result<(), Error> {
        let mut environment = self.facts.generics.environments.get(&owner).cloned();
        self.trait_environment = None;
        self.canonical_trait_inference = None;
        // Body calls express caller universals as free aliases. Add the same
        // assumptions in that context before either query cache is constructed.
        // Original subjects remain available for declaration-context annotations.
        let mut predicates = self
            .facts
            .generics
            .predicates_in_scope(owner)
            .filter(|p| !matches!(p.value.bound, Bound::Outlives(_)))
            .filter(|p| {
                p.value
                    .escaping_binders(&self.facts.types)
                    .is_ok_and(|n| n != 0)
            })
            .map(|p| p.value)
            .collect::<Vec<_>>();
        if !predicates.is_empty() {
            for predicate in &mut predicates {
                *predicate = self.alias_predicate(node, *predicate)?;
            }
            let original = environment
                .as_ref()
                .ok_or_else(|| self.error(node, "missing bounded body environment"))?;
            environment = Some(std::sync::Arc::new(
                original
                    .as_ref()
                    .clone()
                    .with_predicates(&self.facts.types, &predicates)
                    .map_err(|e| self.error(node, e))?,
            ));
            if let Some(environment) = &environment {
                self.facts
                    .generics
                    .body_environments
                    .insert(owner, std::sync::Arc::clone(environment));
            }
        }
        self.method_environment.clone_from(&environment);
        self.parameter_shapes = environment
            .as_ref()
            .filter(|e| e.has_shape_assumptions())
            .map(|e| crate::types::parameters::shape::Shapes::new(std::sync::Arc::clone(e)));
        self.parameter_copies = environment.map(crate::types::copy::Copies::in_environment);
        Ok(())
    }
    pub(super) fn body_shape(&mut self, node: NodeId, ty: TypeId) -> Result<Properties, Error> {
        if self.facts.types.is_sized(ty) == Some(true)
            && self.facts.types.is_well_formed(ty) == Some(true)
        {
            return Ok(Properties {
                sized: true,
                well_formed: true,
            });
        }
        if let Some(shapes) = &mut self.parameter_shapes {
            shapes
                .demand(&self.facts.types, ty)
                .map_err(|e| self.error(node, e))
        } else {
            Ok(Properties {
                sized: self
                    .facts
                    .types
                    .is_sized(ty)
                    .ok_or_else(|| self.error(node, "invalid sizedness type"))?,
                well_formed: self
                    .facts
                    .types
                    .is_well_formed(ty)
                    .ok_or_else(|| self.error(node, "invalid well-formedness type"))?,
            })
        }
    }
    pub(super) fn prove_bound(
        &mut self,
        node: NodeId,
        ty: TypeId,
        bound: Bound,
    ) -> Result<(), Error> {
        if let Bound::Outlives(shorter) = bound {
            self.facts
                .type_outlives
                .push(crate::sema::model::TypeOutlives {
                    body: self.body,
                    source: self.key(node),
                    ty,
                    shorter,
                });
            return Ok(());
        }
        if let Bound::Trait(application) = bound {
            if self
                .method_environment
                .as_ref()
                .is_some_and(|environment| environment.assumes_trait(ty, application))
            {
                return Ok(());
            }
            return self.prove_canonical_trait(
                node,
                crate::types::parameters::predicates::Predicate { ty, bound },
            );
        }
        let valid = if bound == Bound::Sized {
            self.body_shape(node, ty)?.sized
        } else {
            if self.facts.generics.has_nominal_requirements {
                self.facts
                    .aggregates
                    .prepare(&mut self.facts.types, ty)
                    .map_err(|e| self.error(node, e))?;
            }
            let properties = self
                .parameter_copies
                .as_mut()
                .unwrap_or(&mut self.facts.copies)
                .demand(
                    &self.facts.types,
                    &self.facts.aggregates,
                    &self.facts.derives,
                    ty,
                )
                .map_err(|e| self.error(node, e))?;
            if bound == Bound::Copy {
                properties.copy
            } else {
                properties.clone_
            }
        };
        if valid {
            Ok(())
        } else {
            Err(self.error(
                node,
                match bound {
                    Bound::Copy => "generic argument must implement Copy",
                    Bound::Clone => "generic argument must implement Clone",
                    Bound::Sized => "generic argument must be sized",
                    Bound::Trait(_) => "generic argument must implement its declared trait",
                    Bound::Outlives(_) => "generic argument must outlive its bound",
                },
            ))
        }
    }
    pub(super) fn ordinary_bound_obligations(
        &mut self,
        node: NodeId,
        owner: DefId,
        ty: TypeId,
    ) -> Result<(), Error> {
        if !self.facts.generics.predicates.contains_key(&owner) {
            return Ok(());
        }
        let Some(Kind::FunctionItem { arguments, .. }) = self.facts.types.kind(ty) else {
            return Ok(());
        };
        let arguments = self
            .facts
            .types
            .region_args(arguments)
            .ok_or_else(|| self.error(node, "invalid item lifetime arguments"))?
            .iter()
            .copied()
            .map(Argument::Lifetime)
            .collect::<Vec<_>>();
        self.bound_obligations(node, owner, &arguments, &[])
    }
    pub(super) fn bound_obligations(
        &mut self,
        node: NodeId,
        owner: DefId,
        arguments: &[Argument],
        bindings: &[(ParameterId, Argument)],
    ) -> Result<(), Error> {
        let predicates = self
            .facts
            .generics
            .predicates_in_scope(owner)
            .copied()
            .collect::<Vec<_>>();
        if predicates.is_empty() {
            return Ok(());
        }
        let mut regions = Vec::new();
        if let Some(scheme) = self.facts.lifetimes.functions.get(&owner) {
            regions.resize(scheme.total as usize, crate::types::RegionId::STATIC);
            let captures = scheme.captures as usize;
            for (slot, argument) in regions.iter_mut().take(captures).zip(arguments) {
                let Argument::Lifetime(region) = argument else {
                    return Err(self.error(node, "missing captured bound argument"));
                };
                *slot = *region;
            }
            for ((_, index), argument) in scheme.early.iter().zip(arguments.iter().skip(captures)) {
                let Argument::Lifetime(region) = argument else {
                    return Err(self.error(node, "missing early bound argument"));
                };
                regions[usize::from(*index)] = *region;
            }
        }
        let mut substitution = substitute::Bound::new(regions);
        let mut inference =
            crate::types::infer::Instantiation::new(&mut self.infer, &self.facts.types, bindings)
                .map_err(|e| self.error(node, e))?;
        for predicate in predicates {
            let predicate = substitution
                .predicate(&mut self.facts.types, predicate.value)
                .map_err(|e| self.error(node, e))?;
            let predicate = self
                .infer
                .instantiate_predicate(&mut self.facts.types, predicate, &mut inference)
                .map_err(|e| self.error(node, e))?;
            self.call_types.obligations.push((node, predicate));
        }
        Ok(())
    }
}
