//! Projection queries retain both source proof scopes and callable scopes introduced by types.
mod proofs;
use super::{Checker, Error, NodeKey, Request, aliases, regions};
use crate::support::hash::HashMap;
use crate::types::{
    lifetime_proof::{ScopeId, ScopePlan, map_scopes},
    parameters::environment::Environment,
    substitute::{Bound, Normalizer},
};
use std::sync::Arc;

#[derive(Clone, Copy, Default)]
pub(super) enum Ambient {
    #[default]
    Root,
    Alias {
        plan: usize,
        scope: ScopeId,
    },
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sink {
    Declaration(usize),
    Body(usize),
}
#[derive(Default)]
pub(super) struct Mapping {
    sink: Option<Sink>,
    bindings: Vec<ScopeId>,
    environments: HashMap<u16, Option<Arc<Environment>>>,
}
pub(super) struct Plan {
    source: ScopePlan,
    sink: Option<Sink>,
    bindings: Vec<ScopeId>,
}
impl Plan {
    pub fn new(source: ScopePlan) -> Self {
        Self {
            source,
            sink: None,
            bindings: Vec::new(),
        }
    }
}
impl Checker<'_> {
    pub(super) fn ambient_depth(&self, key: NodeKey, ambient: Ambient) -> Result<u16, Error> {
        match ambient {
            Ambient::Root => Ok(0),
            Ambient::Alias { plan, scope } => self
                .projections
                .plans
                .get(plan)
                .ok_or_else(|| self.error(key, "missing alias normalization scope plan"))?
                .source
                .depth(scope)
                .map_err(|e| self.error(key, e)),
        }
    }
    pub(super) fn scoped_projection_environment(
        &mut self,
        key: NodeKey,
        request: Request,
        context: regions::Context,
    ) -> Result<(Option<Arc<Environment>>, u32), Error> {
        let query = self
            .projections
            .roots
            .get(&key)
            .ok_or_else(|| self.error(key, "missing scoped projection query"))?;
        let depth = self
            .ambient_depth(key, query.ambient)?
            .checked_add(request.depth)
            .ok_or_else(|| self.error(key, "projection scope depth exhausted"))?;
        if depth == 0 {
            return self.projection_environment(key, context);
        }
        if let Some(environment) = query.scopes.environments.get(&depth) {
            return Ok((
                environment.clone(),
                self.projection_region_start(key, context)?,
            ));
        }
        let (environment, first) = self.projection_environment(key, context)?;
        let environment = environment
            .map(|env| {
                env.substitute_regions(&mut self.facts.types, &mut Bound::shift(depth))
                    .map(Arc::new)
            })
            .transpose()
            .map_err(|e| self.error(key, e))?;
        self.projections
            .roots
            .get_mut(&key)
            .ok_or_else(|| Error::at(key.file, 0, "missing scoped projection query"))?
            .scopes
            .environments
            .insert(depth, environment.clone());
        Ok((environment, first))
    }

    pub(super) fn projection_alias_request_scope(
        &mut self,
        key: NodeKey,
        request: Request,
        context: regions::Context,
        template: aliases::Template,
    ) -> Result<ScopeId, Error> {
        let root = self.projection_alias_scope(key, context, template)?;
        if request.scope == ScopeId::ROOT {
            return Ok(root);
        }
        let query = self
            .projections
            .roots
            .get_mut(&key)
            .ok_or_else(|| Error::at(key.file, 0, "missing alias projection query"))?;
        let requirements = &mut self.facts.alias_requirements;
        map_binders(
            &mut query.scopes.bindings,
            &query.fold,
            root,
            request.scope,
            |parent, count| {
                let scope = requirements.scope(template.owner, template.parameters, parent)?;
                requirements.complete_scope(template.owner, scope, count);
                Ok(scope)
            },
        )
        .map_err(|e| self.error(key, e))
    }
}

fn map_binders(
    bindings: &mut Vec<ScopeId>,
    fold: &Normalizer,
    root: ScopeId,
    needed: ScopeId,
    mut create: impl FnMut(ScopeId, u32) -> Result<ScopeId, &'static str>,
) -> Result<ScopeId, &'static str> {
    map_scopes(
        bindings,
        root,
        needed,
        |id| fold.binder(id).ok_or("missing normalization binder"),
        &mut create,
    )
}
