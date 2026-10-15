//! Unproved alias type formation is discharged with the actual use's completed environment.
use super::{Checker, Destination, Error, NodeKey, Request, Resolution, Type, TypeId, regions};
use crate::{resolve::model::DefKind, types::parameters::predicates::Predicate};

impl Checker<'_> {
    pub(super) fn alias_parameter_environment(
        &mut self,
        key: NodeKey,
        owner: super::DefId,
    ) -> Result<std::sync::Arc<crate::types::parameters::environment::Environment>, Error> {
        if let Some(environment) = self.projections.alias_environments.get(&owner) {
            return Ok(std::sync::Arc::clone(environment));
        }
        let environment = std::sync::Arc::new(
            self.facts
                .generics
                .parameter_scope(&self.facts.types, owner)
                .map_err(|e| self.error(key, e))?,
        );
        self.projections
            .alias_environments
            .insert(owner, std::sync::Arc::clone(&environment));
        Ok(environment)
    }
    pub(super) fn may_defer_projection(
        &self,
        key: NodeKey,
        context: regions::Context,
        predicate: Predicate,
    ) -> Result<bool, Error> {
        let Some(template) = self.projection_alias_owner(context) else {
            return Ok(false);
        };
        let owner = template.parameters;
        if self.report.resolution.definitions[owner.0 as usize].kind == DefKind::TypeAlias
            && self.facts.associated.owner(owner).is_none()
        {
            return Ok(true);
        }
        if template.owner != template.parameters {
            return Ok(predicate
                .contains_parameters(&self.facts.types)
                .map_err(|e| self.error(key, e))?
                || predicate
                    .has_free_regions(&self.facts.types)
                    .map_err(|e| self.error(key, e))?);
        }
        Ok(false)
    }

    pub(super) fn defer_projection_requirement(
        &mut self,
        key: NodeKey,
        request: Request,
    ) -> Result<(), Error> {
        let context = self
            .projections
            .roots
            .get(&key)
            .ok_or_else(|| self.error(key, "missing deferred projection query"))?
            .context;
        let template = self
            .projection_alias_owner(context)
            .ok_or_else(|| self.error(key, "deferred projection requires an alias contract"))?;
        let scope = self.projection_alias_request_scope(key, request, context, template)?;
        let ty = self.intern(key, Type::Projection(request.application))?;
        self.facts
            .alias_requirements
            .normalization(template.owner, template.parameters, ty, scope)
            .map_err(|e| self.error(key, e))?;
        let query = self
            .projections
            .roots
            .get_mut(&key)
            .ok_or_else(|| Error::at(key.file, 0, "missing deferred projection query"))?;
        let step = query
            .fold
            .resume(&mut self.facts.types, request, Resolution::Deferred)
            .map_err(|e| self.error(key, e))?;
        self.advance_projection(key, step)
    }

    pub(in super::super) fn complete_alias_normalizations(
        &mut self,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        loop {
            if let Some((key, context, ambient, ty)) = self.projections.deferred.pop() {
                self.start_scoped_normalization(
                    key,
                    ty,
                    context,
                    Destination::Requirement,
                    ambient,
                )?;
                self.drain(steps, limit)?;
            } else if let Some((key, context, ambient, ty)) = self.projections.sized.pop() {
                if self.facts.types.contains_projections(ty) == Some(true) {
                    self.start_scoped_normalization(key, ty, context, Destination::Sized, ambient)?;
                    self.drain(steps, limit)?;
                } else {
                    let depth = self.ambient_depth(key, ambient)?;
                    self.prove_alias_size(key, context, depth, ty)?;
                }
            } else {
                return Ok(());
            }
        }
    }

    pub(super) fn retain_alias_sizes(
        &mut self,
        key: NodeKey,
        request: Request,
        context: regions::Context,
        sized: Vec<TypeId>,
        mut restore: Option<&mut crate::types::substitute::Bound>,
    ) -> Result<(), Error> {
        if sized.is_empty() {
            return Ok(());
        }
        let template = self
            .projection_alias_owner(context)
            .ok_or_else(|| self.error(key, "missing alias premise owner"))?;
        for ty in sized {
            if self.facts.types.contains_parameters(ty) != Some(true) {
                continue;
            }
            let ty = if let Some(restore) = &mut restore {
                restore
                    .apply(&mut self.facts.types, ty)
                    .map_err(|e| self.error(key, e))?
            } else {
                ty
            };
            let scope = self.projection_alias_request_scope(key, request, context, template)?;
            self.facts
                .alias_requirements
                .sized(template.owner, template.parameters, ty, scope)
                .map_err(|e| self.error(key, e))?;
        }
        Ok(())
    }

    pub(super) fn prove_alias_size(
        &mut self,
        key: NodeKey,
        context: regions::Context,
        depth: u16,
        ty: TypeId,
    ) -> Result<(), Error> {
        let (mut environment, _) = self.projection_environment(key, context)?;
        if depth != 0
            && let Some(existing) = environment
        {
            environment = Some(std::sync::Arc::new(
                existing
                    .substitute_regions(
                        &mut self.facts.types,
                        &mut crate::types::substitute::Bound::shift(depth),
                    )
                    .map_err(|e| self.error(key, e))?,
            ));
        }
        // Sized has no lifetime witnesses; only captured type assumptions need shifting.
        let mut proof = crate::sema::nominals::Proof::new(environment);
        if !proof
            .prove(
                &mut self.facts,
                Predicate {
                    ty,
                    bound: crate::types::parameters::predicates::Bound::Sized,
                },
            )
            .map_err(|e| self.error(key, e))?
        {
            return Err(self.error(key, "alias projection requires a Sized type argument"));
        }
        Ok(())
    }
}
