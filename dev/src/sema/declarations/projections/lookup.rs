//! Written parameter bounds name members before the caller's proof environment is complete.
mod headers;
use super::{Checker, DefId, Error, NodeKey, Work, regions};
use crate::sema::declarations::generics::bounds::lexical::Scope;
use crate::support::hash::{HashMap, HashSet};
use crate::types::{
    parameters::{
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
    substitute,
};

#[derive(Clone, Copy)]
pub(in super::super) struct Seed {
    owner: DefId,
    subject: DefId,
    path: NodeKey,
    definition: DefId,
    context: regions::Context,
    scope: Option<Scope>,
}

#[derive(Default)]
pub(super) struct Pending {
    headers: Option<Box<headers::Headers>>,
    pub(super) members: HashMap<
        (Predicate, crate::resolve::names::NameId),
        Option<crate::types::projection::Application>,
    >,
    bounds: HashMap<(DefId, DefId), Vec<Seed>>,
    typed: HashMap<NodeKey, Clause>,
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
        scope: Option<Scope>,
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
                scope,
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
        let subject = if seed.scope.is_some() {
            self.capture_bound_type(seed.path, subject)?
        } else {
            subject
        };
        let arguments = self.application_arguments(seed.path, seed.definition, seed.context)?;
        let application = self.applied_trait(seed.path, seed.definition, subject, arguments)?;
        let clause = self.bound_clause(
            seed.path,
            seed.scope,
            Predicate {
                ty: subject,
                bound: Bound::Trait(application),
            },
        )?;
        self.projections.lookup.typed.insert(seed.path, clause);
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
        let mut opened = HashMap::default();
        for seed in seeds {
            let predicate = *self
                .projections
                .lookup
                .typed
                .get(&seed.path)
                .ok_or_else(|| self.error(key, "unfinished shorthand bound inputs"))?;
            let source = if seed.scope.is_some() {
                seed.context
                    .binder
                    .and_then(|binder| self.regions.parent(binder))
            } else {
                seed.context.binder
            };
            let predicate = if matches!(predicate, Clause::ForAll(_)) {
                let identity = (predicate, source);
                if let Some(&value) = opened.get(&identity) {
                    value
                } else {
                    let value = self.projection_naming_predicate(key, predicate, Some(context))?;
                    opened.insert(identity, value);
                    value
                }
            } else {
                predicate.ordinary().map_err(|e| self.error(key, e))?
            };
            result.push(self.capture_projection_predicate(key, predicate, source, context)?);
        }
        Ok(result)
    }

    /// Local annotations may infer an application of a quantified bound. Item
    /// signatures need explicit trait arguments; unused locals can be removed.
    fn projection_naming_predicate(
        &mut self,
        key: NodeKey,
        clause: Clause,
        context: Option<regions::Context>,
    ) -> Result<Predicate, Error> {
        match clause {
            Clause::Predicate(predicate) => Ok(predicate),
            Clause::ForAll(id) => {
                let value = self
                    .facts
                    .types
                    .quantified(id)
                    .ok_or_else(|| self.error(key, "missing shorthand bound quantifier"))?;
                if let Some(context) = context.filter(|context| {
                    matches!(
                        context.position,
                        regions::Position::Local(_) | regions::Position::Constant(_)
                    )
                }) {
                    let mut locals = Vec::with_capacity(value.bound_regions as usize);
                    for _ in 0..value.bound_regions {
                        locals.push(self.region(key, None, context)?);
                    }
                    return substitute::Bound::new(locals)
                        .predicate(&mut self.facts.types, value.predicate)
                        .map_err(|e| self.error(key, e));
                }
                substitute::Bound::lower(1)
                    .predicate(&mut self.facts.types, value.predicate)
                    .map_err(|_| {
                        self.error(
                            key,
                            "associated type requires explicit higher-ranked trait qualification",
                        )
                    })
            }
        }
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
