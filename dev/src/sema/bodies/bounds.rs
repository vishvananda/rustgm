//! Body obligations share the same fixed assumptions as repeat and shape queries.
mod permissions;
mod universal;
use super::{Checker, DefId, Error, NodeId, TypeId};
use crate::types::{
    Kind,
    infer::arguments::Argument,
    parameters::{ParameterId, predicates::Bound, shape::Properties},
    substitute,
};
pub(super) use universal::Requirements;
impl super::State {
    pub(super) fn prepare_parameter_queries(&mut self) {
        self.parameter_shapes = self
            .method_environment
            .as_ref()
            .filter(|e| e.has_shape_assumptions())
            .map(|e| crate::types::parameters::shape::Shapes::new(std::sync::Arc::clone(e)));
        self.parameter_copies = self
            .method_environment
            .as_ref()
            .map(|e| crate::types::copy::Copies::in_environment(std::sync::Arc::clone(e)));
    }
}
impl Checker<'_> {
    pub(super) fn body_is_sized(&mut self, node: NodeId, ty: TypeId) -> Result<bool, Error> {
        let sized = self.body_shape(node, ty)?.sized;
        if sized
            && self.facts.types.nominal_sizes_pending()
            && self.facts.types.contains_nominals(ty) == Some(true)
        {
            let requirement = crate::sema::nominals::sizes::Requirement {
                body: self.state.body,
                source: self.key(node),
                ty,
                environment: self.state.method_environment.clone(),
            };
            self.facts
                .generics
                .size_rules
                .get_or_insert_with(Default::default)
                .require(requirement)
                .map_err(|e| self.error(node, e))?;
        }
        Ok(sized)
    }

    pub(super) fn nominal_requirements(
        &mut self,
        node: NodeId,
        ty: TypeId,
        checked: &mut Requirements,
    ) -> Result<(), Error> {
        // Outlives preparation and requirement walks both find work only at
        // nominal instances or projections, which the structural properties cover.
        if self.facts.types.contains_nominals(ty) == Some(false)
            && self.facts.types.contains_projections(ty) == Some(false)
        {
            return Ok(());
        }
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
        signature: Option<TypeId>,
    ) -> Result<(), Error> {
        self.state.environment_owner = Some(owner);
        let declaration_environment = self.facts.generics.environments.get(&owner).cloned();
        self.state.trait_environment = None;
        self.state.canonical_trait_inference = None;
        self.state.canonical_trait_proofs = None;
        self.state.canonical_requirements = None;
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
        if signature.is_some()
            && let Some(clause) = self
                .facts
                .default_self_clause(owner, self.state.body)
                .map_err(|e| self.error(node, e))?
        {
            predicates.push(clause);
        }
        for predicate in &mut predicates {
            *predicate = self.alias_clause(node, *predicate)?;
        }
        let defining =
            crate::sema::function_context::assumption_environment::Prepared::for_declaration(
                self.facts,
                owner,
                signature,
                self.state.body_binders,
                declaration_environment,
                &predicates,
            )
            .map_err(|e| self.error(node, e))?;
        if defining.needs_aliases() {
            self.prepare_universals(node)?;
        }
        let aliases = self.state.call_types.universal_arguments();
        if defining.has_environment() {
            self.facts.body_work.canonical_steps = self
                .facts
                .body_work
                .canonical_steps
                .saturating_add(aliases.len());
        }
        let completed = defining
            .finish(self.state.body_binders, aliases)
            .map_err(|e| self.error(node, e))?;
        let environment = completed.environment;
        if completed.changed
            && let Some(environment) = &environment
        {
            self.facts
                .generics
                .body_environments
                .insert(owner, std::sync::Arc::clone(environment));
        }
        self.state.method_environment.clone_from(&environment);
        self.state.prepare_parameter_queries();
        if self.facts.source_bindings.is_some() {
            let declaration = crate::sema::compiler_items::copy_declaration(self.facts)
                .map_err(|e| self.error(node, e))?;
            let requirements = self
                .facts
                .generics
                .builtin_requirements
                .entry(self.state.body)
                .or_insert_with(|| crate::sema::model::BuiltinRequirements {
                    environment: environment.clone(),
                    predicates: Vec::new(),
                    source_copy: None,
                });
            requirements.environment.clone_from(&environment);
            requirements.source_copy = Some(std::sync::Arc::new(
                crate::sema::model::SourceCopyRequirements {
                    declaration,
                    goals: Vec::new(),
                },
            ));
        }
        Ok(())
    }
    pub(super) fn body_shape(&mut self, node: NodeId, ty: TypeId) -> Result<Properties, Error> {
        if self.facts.types.contains_projected_sizes(ty) == Some(true) {
            return self.projected_storage_shape(node, ty);
        }
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
    fn projected_storage_shape(&mut self, node: NodeId, ty: TypeId) -> Result<Properties, Error> {
        let ty = self.alias_type(node, ty)?;
        let first = self
            .facts
            .body_regions
            .frontier(self.state.body)
            .map_err(|_| self.error(node, "too many body proof regions"))?;
        let mut proof = self.state.nominal_shapes.take().unwrap_or_else(|| {
            Box::new(crate::sema::nominals::Proof::for_declaration(
                self.state.method_environment.clone(),
                0,
                first,
            ))
        });
        proof
            .resume_body_shapes(first)
            .map_err(|e| self.error(node, e))?;
        let result = proof
            .check(self.facts, ty)
            .map_err(|e| self.error(node, e))?;
        if let Some(evidence) = proof.take_trait_proof() {
            evidence
                .retain_body(self.facts, self.state.body, self.key(node))
                .map_err(|e| self.error(node, e))?;
        }
        self.state.nominal_shapes = Some(proof);
        Ok(result)
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
        if bound.trait_parts().is_some() {
            if self
                .state
                .method_environment
                .as_ref()
                .is_some_and(|environment| {
                    environment.assumes_predicate(crate::types::parameters::predicates::Predicate {
                        ty,
                        bound,
                    })
                })
            {
                return Ok(());
            }
            return self.prove_canonical_trait(
                node,
                crate::types::parameters::predicates::Predicate { ty, bound },
            );
        }
        if matches!(bound, Bound::Auto(_)) {
            return self.prove_canonical_trait(
                node,
                crate::types::parameters::predicates::Predicate { ty, bound },
            );
        }
        let valid = if bound == Bound::Sized {
            self.body_is_sized(node, ty)?
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
        if valid || self.defer_builtin_identity(node, ty, bound)? {
            Ok(())
        } else {
            Err(self.error(
                node,
                match bound {
                    Bound::Copy => "generic argument must implement Copy",
                    Bound::Clone => "generic argument must implement Clone",
                    Bound::Sized => "generic argument must be sized",
                    Bound::Auto(_) => "generic argument must implement its auto trait",
                    Bound::Trait(_) | Bound::ConstTrait(_) | Bound::MaybeConstTrait(_) => {
                        "generic argument must implement its declared trait"
                    }
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
        let predicates = crate::sema::declaration_inputs::all(
            &self.facts.generics,
            self.facts.crates.as_deref(),
            owner,
        )
        .map_err(|e| self.error(node, e))?
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
        let mut inference = crate::types::infer::Instantiation::pooled(
            &mut self.state.infer,
            &self.facts.types,
            bindings,
            &mut self.state.call_types.instantiations,
        )
        .map_err(|e| self.error(node, e))?;
        // Equivalent requirements under this one call substitution need one proof.
        // Keep declaration clauses intact and retain the first clause's own binder.
        let mut quantified = crate::support::hash::HashSet::default();
        for predicate in predicates {
            let effect = self.callee_use_effect()?;
            let copied = self
                .facts
                .member_use_clause(owner, predicate, effect)
                .map_err(|e| self.error(node, e))?;
            let clause =
                crate::sema::traits::effects::in_context(&mut self.facts.types, copied, effect)
                    .map_err(|e| self.error(node, e))?;
            if let crate::types::parameters::clauses::Clause::ForAll(id) = clause {
                let (key, work) = self
                    .facts
                    .types
                    .clause_comparison(id)
                    .map_err(|e| self.error(node, e))?;
                self.state
                    .infer
                    .account_work(work.saturating_add(1))
                    .map_err(|e| self.error(node, e))?;
                if !quantified.insert(key) {
                    continue;
                }
            }
            self.instantiate_bound_obligation(
                node,
                clause,
                assumed,
                &mut substitution,
                &mut inference,
            )?;
        }
        inference.recycle(&mut self.state.call_types.instantiations);
        Ok(())
    }

    pub(super) fn callee_use_effect(&self) -> Result<crate::support::constness::Constness, Error> {
        use crate::support::constness::Constness;
        if self.runtime_callee_context()? {
            return Ok(Constness::Ordinary);
        }
        // Only this actual closed initializer has unconditional const demand.
        // Defining generic const bodies and inline children retain Conditional.
        if let super::readiness::Completion::Initializer(index) = self.state.completion
            && !self.state.in_function
            && self.state.body_binders == 0
            && let Some(initializer) = self.facts.initializers.get(index)
            && initializer.value == self.state.body
            && !self.facts.generics.has_parameters(initializer.definition)
        {
            return Ok(Constness::Const);
        }
        Ok(Constness::Conditional)
    }

    fn runtime_callee_context(&self) -> Result<bool, Error> {
        if !self.state.in_function {
            return Ok(false);
        }
        // Closures retain the enclosing environment, but own a distinct body.
        let function = self
            .state
            .environment_owner
            .and_then(|owner| self.facts.function_indices.get(&owner))
            .and_then(|&index| self.facts.functions.get(index));
        let Some(function) = function.filter(|f| f.body == Some(self.state.body)) else {
            return Ok(true);
        };
        self.facts
            .source_function_context(self.report, function.definition, self.state.body)
            .map(|context| !context.checks_const())
            .map_err(|e| self.error(self.state.body.node, e))
    }

    pub(super) fn instantiate_bound_obligation(
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
        self.record_closure_expectation(node, &predicate, bound_regions)?;
        if let Some(count) = bound_regions {
            self.state.call_types.quantify(index, count);
        } else {
            self.defer_bound_inputs(node, &predicate)?;
        }
        self.state.call_types.obligations.push((node, predicate));
        Ok(())
    }

    pub(super) fn defer_bound_inputs(
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
            Some(Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_) | Bound::Outlives(_))
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
        let trait_owner = self.facts.associated.trait_owner(owner);
        if trait_owner.is_some_and(|owner| {
            self.facts
                .cloning
                .as_deref()
                .is_some_and(|r| r.owns_trait(owner))
        }) {
            // Clone bounds are scoped marker premises. The actual canonical
            // application must use selection's marker bridge, not pretend that
            // the caller supplied an explicit user-trait assumption.
            return Ok(None);
        }
        trait_owner
            .and_then(|owner| self.facts.generics.trait_self_bounds.get(&owner))
            .copied()
            .map(Some)
            .ok_or_else(|| self.error(node, "bound-selected method has no trait Self predicate"))
    }
}
