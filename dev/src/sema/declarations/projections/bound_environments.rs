//! Typed bound subjects form assumptions before their projections are normalized.
use super::{Checker, DefId, Error, NodeKey, Work, regions};
use crate::sema::declarations::generics::bounds::Syntax;
use std::{collections::HashSet, sync::Arc};

pub(super) struct Bounds {
    syntax: Vec<Syntax>,
    roots: Vec<NodeKey>,
}
impl Checker<'_> {
    pub(in super::super) fn defer_bound_projection(&mut self, key: NodeKey) {
        self.projections.raw_bound_nodes.insert(key);
    }
    pub(in super::super) fn bound_environment_pending(&self, owner: DefId) -> bool {
        self.projections.raw_bounds.contains_key(&owner)
    }
    pub(in super::super) fn retain_raw_bounds(
        &mut self,
        owner: DefId,
        syntax: Vec<Syntax>,
    ) -> Result<(), Error> {
        let mut roots = Vec::new();
        let mut seen = HashSet::new();
        for key in syntax.iter().filter_map(|s| s.subject()) {
            if self.facts.types.contains_projections(self.ty(key)?) == Some(true)
                && seen.insert(key)
            {
                roots.push(key);
            }
        }
        if !roots.is_empty() {
            self.projections
                .raw_bounds
                .insert(owner, Bounds { syntax, roots });
        }
        Ok(())
    }

    pub(super) fn prepare_bound_environment(
        &mut self,
        owner: DefId,
        context: regions::Context,
    ) -> Result<bool, Error> {
        if self.projections.raw_environments.contains_key(&owner) {
            return Ok(false);
        }
        let Some(bounds) = self.projections.raw_bounds.get(&owner) else {
            return Ok(false);
        };
        let roots = bounds.roots.clone();
        let key = roots[0];
        let environment = self
            .facts
            .generics
            .elaborated_environment(&mut self.facts.types, owner)
            .map_err(|e| self.error(key, e))?;
        self.projections
            .raw_environments
            .insert(owner, Arc::new(environment));
        self.work
            .push(Work::ProjectionEnvironment(owner, context, true));
        for key in roots.into_iter().rev() {
            self.work.push(Work::NormalizeBound(key, context));
        }
        Ok(true)
    }

    pub(super) fn complete_bound_environment(
        &mut self,
        owner: DefId,
        context: regions::Context,
    ) -> Result<(), Error> {
        if let Some(bounds) = self.projections.raw_bounds.remove(&owner) {
            self.bound_syntax.insert(owner, bounds.syntax);
            self.complete_bounds(owner, context)?;
        }
        self.projections.raw_environments.remove(&owner);
        Ok(())
    }
}
