//! Body obligations share the same fixed assumptions as repeat and shape queries.
mod universal;
use super::{Checker, DefId, Error, NodeId, TypeId};
use crate::types::{
    Kind,
    infer::arguments::Argument,
    parameters::{ParameterId, predicates::Bound, shape::Properties},
    substitute,
};
pub(super) use universal::Requirements;
impl Checker<'_> {
    pub(super) fn nominal_requirements(
        &mut self,
        node: NodeId,
        ty: TypeId,
        checked: &mut Requirements,
    ) -> Result<(), Error> {
        self.facts
            .lifetimes
            .nominal_requirements
            .prepare(&mut self.facts.types, ty)
            .map_err(|e| self.error(node, e))?;
        if !self.facts.generics.has_nominal_requirements {
            return Ok(());
        }
        let requirements =
            crate::sema::nominals::requirements(self.facts, ty, &mut checked.ordinary)
                .map_err(|e| self.error(node, e))?;
        for requirement in requirements {
            match requirement {
                crate::types::parameters::clauses::Clause::Predicate(requirement) => {
                    self.prove_bound(node, requirement.ty, requirement.bound)?;
                }
                crate::types::parameters::clauses::Clause::ForAll(_) => {
                    self.prove_quantified_requirement(node, requirement, checked)?;
                }
            }
        }
        Ok(())
    }
    pub(super) fn prepare_bound_environment(
        &mut self,
        node: NodeId,
        owner: DefId,
    ) -> Result<(), Error> {
        self.state.environment_owner = Some(owner);
        let mut environment = self.facts.generics.environments.get(&owner).cloned();
        self.state.trait_environment = None;
        self.state.canonical_trait_inference = None;
        // Body calls express caller universals as free aliases. Add the same
        // assumptions in that context before either query cache is constructed.
        // Original subjects remain available for declaration-context annotations.
        let mut predicates = Vec::new();
        for predicate in self.facts.generics.predicates_in_scope(owner) {
            if matches!(
                predicate.value,
                crate::types::parameters::clauses::Clause::Predicate(
                    crate::types::parameters::predicates::Predicate {
                        bound: Bound::Outlives(_),
                        ..
                    }
                )
            ) {
                continue;
            }
            if predicate
                .value
                .escaping_binders(&self.facts.types)
                .map_err(|error| self.error(node, error))?
                != 0
            {
                predicates.push(predicate.value);
            }
        }
        if !predicates.is_empty() {
            for predicate in &mut predicates {
                *predicate = self.alias_clause(node, *predicate)?;
            }
            let original = environment
                .as_ref()
                .ok_or_else(|| self.error(node, "missing bounded body environment"))?;
            environment = Some(std::sync::Arc::new(
                original
                    .as_ref()
                    .clone()
                    .with_clauses(&self.facts.types, &predicates)
                    .map_err(|e| self.error(node, e))?,
            ));
            if let Some(environment) = &environment {
                self.facts
                    .generics
                    .body_environments
                    .insert(owner, std::sync::Arc::clone(environment));
            }
        }
        self.state.method_environment.clone_from(&environment);
        self.state.parameter_shapes = environment
            .as_ref()
            .filter(|e| e.has_shape_assumptions())
            .map(|e| crate::types::parameters::shape::Shapes::new(std::sync::Arc::clone(e)));
        self.state.parameter_copies = environment.map(crate::types::copy::Copies::in_environment);
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
        let projected = self.facts.types.contains_projections(ty) == Some(true);
        if let Some(shapes) = &mut self.state.parameter_shapes
            && let Some(properties) = shapes
                .try_demand(&self.facts.types, ty)
                .map_err(|e| self.error(node, e))?
            && (!projected || properties.sized && properties.well_formed)
        {
            return Ok(properties);
        }
        let ty = if projected {
            self.normalize_known_projection(node, ty)?
        } else {
            ty
        };
        if let Some(shapes) = &mut self.state.parameter_shapes {
            (if projected {
                shapes.normalized(&self.facts.types, ty)
            } else {
                shapes.demand(&self.facts.types, ty)
            })
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
        if let Bound::ProjectionEqual(result) = bound {
            return self.prove_associated_equality(node, ty, result);
        }
        if let Bound::Outlives(shorter) = bound {
            self.facts
                .type_outlives
                .push(crate::sema::model::TypeOutlives {
                    body: self.state.body,
                    source: self.key(node),
                    ty,
                    shorter,
                });
            return Ok(());
        }
        if let Bound::Trait(application) = bound {
            if self
                .state
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
                self.prepare_storage(node, ty)?;
            }
            let properties = self
                .state
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
                    Bound::ProjectionEqual(_) => "associated type must equal its required result",
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
        if !self.facts.lifetimes.functions.contains_key(&owner)
            && let Some(schema) = self.facts.associated.schema(owner)
        {
            for argument in arguments.iter().take(schema.captures as usize) {
                let Argument::Lifetime(region) = *argument else {
                    return Err(self.error(node, "missing constant owner lifetime argument"));
                };
                regions.push(region);
            }
        }
        let assumed = self.bound_method_requirement(node, owner)?;
        let mut substitution = substitute::Bound::new(regions);
        let mut inference = crate::types::infer::Instantiation::new(
            &mut self.state.infer,
            &self.facts.types,
            bindings,
        )
        .map_err(|e| self.error(node, e))?;
        for predicate in predicates {
            self.instantiate_bound_obligation(
                node,
                predicate.value,
                assumed,
                &mut substitution,
                &mut inference,
            )?;
        }
        Ok(())
    }

    fn instantiate_bound_obligation(
        &mut self,
        node: NodeId,
        clause: crate::types::parameters::clauses::Clause,
        assumed: Option<crate::types::parameters::predicates::Predicate>,
        substitution: &mut substitute::Bound,
        inference: &mut crate::types::infer::Instantiation,
    ) -> Result<(), Error> {
        use crate::types::parameters::clauses::Clause;
        let index = self.state.call_types.obligations.len();
        if assumed.is_some_and(|value| clause == Clause::Predicate(value)) {
            self.state.call_types.assumed_obligations.insert(index);
        }
        let clause = substitution
            .clause(&mut self.facts.types, clause)
            .map_err(|e| self.error(node, e))?;
        let (value, bound_regions) = match clause {
            Clause::Predicate(value) => (value, None),
            Clause::ForAll(id) => {
                let value = self
                    .facts
                    .types
                    .quantified(id)
                    .ok_or_else(|| self.error(node, "missing quantified call requirement"))?;
                (value.predicate, Some(value.bound_regions))
            }
        };
        let predicate = self
            .state
            .infer
            .instantiate_predicate(&mut self.facts.types, value, inference)
            .map_err(|e| self.error(node, e))?;
        if let Some(count) = bound_regions {
            self.state.call_types.quantify(index, count);
        } else {
            self.defer_bound_inputs(node, &predicate)?;
        }
        self.state.call_types.obligations.push((node, predicate));
        Ok(())
    }

    fn defer_bound_inputs(
        &mut self,
        node: NodeId,
        predicate: &crate::types::infer::predicates::Predicate,
    ) -> Result<(), Error> {
        let scope = crate::types::lifetime_proof::ScopeId::ROOT;
        if let Some(result) = predicate.projection_result() {
            self.defer_normalization(node, scope, predicate.subject())?;
            self.defer_normalization(node, scope, result)?;
            self.deferred_equation(
                node,
                predicate.subject(),
                result,
                crate::types::infer::deferred::Relation::Equality,
            )?;
        } else if matches!(
            predicate.builtin_bound(),
            Some(Bound::Copy | Bound::Clone | Bound::Sized | Bound::Outlives(_))
        ) {
            // The selected result's property does not erase input formation debts.
            self.defer_normalization(node, scope, predicate.subject())?;
        }
        Ok(())
    }

    fn bound_method_requirement(
        &self,
        node: NodeId,
        owner: DefId,
    ) -> Result<Option<crate::types::parameters::predicates::Predicate>, Error> {
        if !self.facts.methods.bound_uses.contains(&self.key(node)) {
            return Ok(None);
        }
        self.facts
            .associated
            .trait_owner(owner)
            .and_then(|owner| self.facts.generics.trait_self_bounds.get(&owner))
            .copied()
            .map(Some)
            .ok_or_else(|| self.error(node, "bound-selected method has no trait Self predicate"))
    }
}
