//! Written parameter bounds name members before the caller's proof environment is complete.
mod headers;
use super::{Checker, DefId, Error, NodeKey, Work, regions};
use crate::types::{
    parameters::predicates::{Bound, Predicate},
    substitute,
};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Copy)]
pub(in super::super) struct Seed {
    owner: DefId,
    subject: DefId,
    path: NodeKey,
    definition: DefId,
    context: regions::Context,
}

#[derive(Default)]
pub(super) struct Pending {
    headers: Option<Box<headers::Headers>>,
    pub(super) members: HashMap<
        (Predicate, crate::resolve::names::NameId),
        Option<crate::types::projection::Application>,
    >,
    bounds: HashMap<(DefId, DefId), Vec<Seed>>,
    typed: HashMap<NodeKey, Predicate>,
    active: HashSet<NodeKey>,
    queries: HashMap<NodeKey, Vec<Seed>>,
}

impl Checker<'_> {
    pub(in super::super) fn retain_projection_bound(
        &mut self,
        owner: DefId,
        subject: DefId,
        path: NodeKey,
        definition: DefId,
        context: regions::Context,
    ) {
        self.projections
            .lookup
            .bounds
            .entry((owner, subject))
            .or_default()
            .push(Seed {
                owner,
                subject,
                path,
                definition,
                context,
            });
    }

    pub(super) fn schedule_projection_bounds(
        &mut self,
        key: NodeKey,
        subject: DefId,
        context: regions::Context,
        name: crate::resolve::names::NameId,
    ) -> Result<(), Error> {
        if self.projections.lookup.queries.contains_key(&key) {
            return Err(self.error(key, "cycle in shorthand associated type inputs"));
        }
        let mut binder = context.binder;
        let mut owner = self
            .projection_alias_owner(context)
            .map(|template| template.parameters);
        while owner.is_none() {
            let Some(current) = binder else { break };
            owner = self.regions.owner(current);
            binder = self.regions.parent(current);
        }
        owner = owner.map(|owner| self.projection_scope_owner(owner));
        let mut seeds = Vec::new();
        while let Some(current) = owner {
            if let Some(bounds) = self.projections.lookup.bounds.get(&(current, subject)) {
                seeds.extend_from_slice(bounds);
            }
            owner = self.facts.generics.parents.get(&current).copied();
        }
        if let Some(headers) = &mut self.projections.lookup.headers {
            headers.reach(name).map_err(|e| Error::at(key.file, 0, e))?;
            seeds.retain(|seed| headers.reachable[&name].contains(&seed.definition));
        }
        for seed in seeds.iter().rev() {
            self.work.push(Work::ProjectionBound(*seed, false));
        }
        self.projections.lookup.queries.insert(key, seeds);
        Ok(())
    }

    pub(in super::super) fn complete_projection_bound(
        &mut self,
        seed: Seed,
        finish: bool,
    ) -> Result<(), Error> {
        if self.projections.lookup.typed.contains_key(&seed.path) {
            return Ok(());
        }
        if !finish {
            if !self.projections.lookup.active.insert(seed.path) {
                return Err(self.error(seed.path, "cycle in shorthand bound arguments"));
            }
            self.work.push(Work::ProjectionBound(seed, true));
            self.bound_subject_lifetimes(seed.path, seed.owner)?;
            self.prepare_associated_bindings(seed.path)?;
            self.work.push(Work::PathArguments(
                seed.path,
                seed.definition,
                seed.context,
            ));
            return Ok(());
        }
        let subject = self.facts.definitions[seed.subject.0 as usize]
            .ok_or_else(|| self.error(seed.path, "missing shorthand bound subject"))?;
        let arguments = self.application_arguments(seed.path, seed.definition, seed.context)?;
        let application = self.applied_trait(seed.path, seed.definition, subject, arguments)?;
        self.projections.lookup.typed.insert(
            seed.path,
            Predicate {
                ty: subject,
                bound: Bound::Trait(application),
            },
        );
        self.projections.lookup.active.remove(&seed.path);
        Ok(())
    }

    pub(super) fn projection_bound_candidates(
        &mut self,
        key: NodeKey,
        context: regions::Context,
    ) -> Result<Vec<Predicate>, Error> {
        let seeds = self
            .projections
            .lookup
            .queries
            .remove(&key)
            .ok_or_else(|| self.error(key, "missing shorthand bound query"))?;
        let mut result = Vec::with_capacity(seeds.len());
        for seed in seeds {
            let predicate = *self
                .projections
                .lookup
                .typed
                .get(&seed.path)
                .ok_or_else(|| self.error(key, "unfinished shorthand bound inputs"))?;
            result.push(self.capture_projection_predicate(
                key,
                predicate,
                seed.context.binder,
                context,
            )?);
        }
        Ok(result)
    }

    pub(super) fn capture_projection_predicate(
        &mut self,
        key: NodeKey,
        predicate: Predicate,
        source: Option<regions::BinderId>,
        context: regions::Context,
    ) -> Result<Predicate, Error> {
        if source == context.binder
            || !predicate
                .has_free_regions(&self.facts.types)
                .map_err(|e| self.error(key, e))?
        {
            return Ok(predicate);
        }
        let source = source.ok_or_else(|| self.error(key, "missing shorthand lifetime binder"))?;
        let mut mapping = Vec::new();
        for index in 0..self.regions.count(source) {
            let slot = u16::try_from(index)
                .map_err(|_| self.error(key, "too many bound lifetime slots"))?;
            mapping.push(
                self.regions
                    .captured_region(context.binder, source, slot)
                    .map_err(|e| self.error(key, e))?,
            );
        }
        substitute::Bound::new(mapping)
            .predicate(&mut self.facts.types, predicate)
            .map_err(|e| self.error(key, e))
    }
}
