//! Body builtin permissions consume declared and implied lifetime assumptions.
use super::{Checker, DefId, Error, NodeId, TypeId};
use crate::types::{
    Kind, arguments::Argument, outlives::Queries, parameters::environment::Environment,
    relations::Constraints,
};

impl Checker<'_> {
    pub(super) fn body_alias_environment(
        &mut self,
        node: NodeId,
        owner: DefId,
        environment: std::sync::Arc<Environment>,
    ) -> Result<std::sync::Arc<Environment>, Error> {
        let aliases = self.state.call_types.universal_arguments();
        if aliases.is_empty() {
            return Ok(environment);
        }
        self.facts.body_work.canonical_steps += aliases.len();
        if aliases.len() != self.state.body_binders as usize {
            return Err(self.error(node, "body aliases do not match the actual defining scope"));
        }
        let environment = std::sync::Arc::new(
            environment
                .as_ref()
                .clone()
                .with_body_aliases(aliases)
                .map_err(|e| self.error(node, e))?,
        );
        self.facts
            .generics
            .body_environments
            .insert(owner, std::sync::Arc::clone(&environment));
        Ok(environment)
    }

    pub(super) fn defer_builtin_identity(
        &mut self,
        node: NodeId,
        ty: TypeId,
        bound: crate::types::parameters::predicates::Bound,
    ) -> Result<bool, Error> {
        let Some(environment) = self
            .state
            .method_environment
            .as_ref()
            .filter(|env| env.has_region_builtins())
        else {
            return Ok(false);
        };
        if self.facts.types.contains_regions(ty) != Some(true) {
            return Ok(false);
        }
        let source = self.key(node);
        let pending = self
            .facts
            .generics
            .builtin_requirements
            .entry(self.state.body)
            .or_insert_with(|| crate::sema::model::BuiltinRequirements {
                environment: Some(std::sync::Arc::clone(environment)),
                predicates: Vec::new(),
                source_copy: None,
            });
        if pending.predicates.len() >= 1_000_000 {
            return Err(self.error(node, "builtin requirement storage budget exceeded"));
        }
        pending.predicates.push((
            source,
            crate::types::parameters::predicates::Predicate { ty, bound },
        ));
        Ok(true)
    }

    pub(super) fn builtin_region_environment(
        &mut self,
        node: NodeId,
        owner: DefId,
        signature: Option<TypeId>,
        environment: Environment,
    ) -> Result<Environment, Error> {
        if !environment.has_region_builtins() {
            return Ok(environment);
        }
        if let Some(signature) = signature {
            self.facts
                .lifetimes
                .nominal_requirements
                .prepare(&mut self.facts.types, signature)
                .map_err(|e| self.error(node, e))?;
        }
        let mut proof = Constraints::new(
            &self.facts.types,
            &self.facts.lifetimes.variances,
            self.state.body_binders,
            1_000_000,
        )
        .map_err(|e| self.error(node, e.to_string()))?;
        for parameter in self.facts.generics.parameters_in_scope(owner) {
            if let Argument::Type(ty) = parameter.argument
                && let Some(Kind::Parameter(id)) = self.facts.types.kind(ty)
            {
                proof
                    .parameter(id)
                    .map_err(|e| self.error(node, e.to_string()))?;
            }
        }
        for &(longer, shorter) in self
            .facts
            .lifetimes
            .functions
            .get(&owner)
            .into_iter()
            .flat_map(|s| &s.bounds)
        {
            proof
                .assume(longer, shorter)
                .map_err(|e| self.error(node, e.to_string()))?;
        }
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        if let Some(signature) = signature {
            proof
                .assume_signature(&mut queries, signature)
                .map_err(|e| self.error(node, e))?;
        }
        proof
            .assume_outlives_clauses(
                &mut queries,
                self.facts
                    .generics
                    .predicates_in_scope(owner)
                    .map(|p| p.value),
            )
            .map_err(|e| self.error(node, e))?;
        let equalities = proof
            .assumed_equalities()
            .map_err(|e| self.error(node, e.to_string()))?;
        if equalities.is_empty() {
            return Ok(environment);
        }
        self.prepare_universals(node)?;
        let equalities = equalities
            .with_aliases(&self.state.call_types.universal_arguments())
            .map_err(|e| self.error(node, e.to_string()))?;
        Ok(environment.with_region_equalities(equalities))
    }
}
