//! Predicate substitution transforms both the subject and the complete trait arguments.
use super::Bound;
use crate::support::hash::HashMap;
use crate::types::{
    Store,
    parameters::{
        clauses::{Clause, Quantified},
        predicates::{Bound as Requirement, Predicate},
    },
    traits::{Application, ApplicationId},
};

#[derive(Default)]
pub(super) struct Applications {
    root: HashMap<ApplicationId, ApplicationId>,
    scoped: Option<Box<HashMap<(ApplicationId, u16), ApplicationId>>>,
}
impl Applications {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.root);
        if let Some(scoped) = &self.scoped {
            total.boxed(scoped.as_ref());
            total.map(scoped);
        }
        total
    }

    fn insertion_growth(&self, depth: u16) -> crate::support::retained::Growth {
        use crate::support::retained::Growth;
        if depth == 0 {
            return Growth::map::<ApplicationId, ApplicationId>(
                self.root.len(),
                self.root.capacity(),
                1,
            );
        }
        if let Some(scoped) = &self.scoped {
            Growth::map::<(ApplicationId, u16), ApplicationId>(scoped.len(), scoped.capacity(), 1)
        } else {
            let mut growth = Growth::map::<(ApplicationId, u16), ApplicationId>(0, 0, 1);
            growth.bytes = growth
                .bytes
                .saturating_add(size_of::<HashMap<(ApplicationId, u16), ApplicationId>>());
            growth
        }
    }

    fn planned_insert(&self, depth: u16) -> usize {
        if depth == 0 {
            crate::support::retained::new_map::<ApplicationId, ApplicationId>(
                self.root.len().saturating_add(1),
            )
        } else {
            size_of::<HashMap<(ApplicationId, u16), ApplicationId>>().saturating_add(
                crate::support::retained::new_map::<(ApplicationId, u16), ApplicationId>(
                    self.scoped
                        .as_ref()
                        .map_or(1, |m| m.len().saturating_add(1)),
                ),
            )
        }
    }
    fn get(&self, id: ApplicationId, depth: u16) -> Option<ApplicationId> {
        if depth == 0 {
            self.root.get(&id).copied()
        } else {
            self.scoped.as_ref()?.get(&(id, depth)).copied()
        }
    }
    fn insert(&mut self, id: ApplicationId, depth: u16, result: ApplicationId) {
        if depth == 0 {
            self.root.insert(id, result);
        } else {
            self.scoped
                .get_or_insert_with(Default::default)
                .insert((id, depth), result);
        }
    }
}

impl Bound {
    pub(crate) fn predicate_charged(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Predicate, &'static str> {
        self.predicate_observed::<true>(store, predicate, 0, charge)
    }
    /// Substitute enclosing arguments without opening a clause's own quantifier.
    /// Both the subject and every requirement input retain the same binder depth.
    /// Opening that quantifier is a separate scoped proof operation.
    /// # Errors
    /// Reports malformed clauses and the same argument/work failures as `predicate`.
    pub fn clause(&mut self, store: &mut Store, clause: Clause) -> Result<Clause, &'static str> {
        self.clause_observed::<false>(store, clause, &|_, _| Ok(()))
    }
    pub(crate) fn clause_charged(
        &mut self,
        store: &mut Store,
        clause: Clause,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Clause, &'static str> {
        self.clause_observed::<true>(store, clause, charge)
    }
    fn clause_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        clause: Clause,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Clause, &'static str> {
        Ok(match clause {
            Clause::Predicate(predicate) => {
                Clause::Predicate(self.predicate_observed::<CHARGED>(store, predicate, 0, charge)?)
            }
            Clause::ForAll(id) => {
                let value = store.quantified(id).ok_or("invalid quantified predicate")?;
                let predicate =
                    self.predicate_observed::<CHARGED>(store, value.predicate, 1, charge)?;
                let value = Quantified { predicate, ..value };
                Clause::ForAll(if CHARGED {
                    store.intern_quantified_charged(value, charge)?
                } else {
                    store.intern_quantified(value)?
                })
            }
        })
    }

    /// Use the same immutable parameter/lifetime mapping as signature substitution.
    /// # Errors
    /// Rejects malformed applications, missing arguments and exhausted fold work.
    pub fn predicate(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
    ) -> Result<Predicate, &'static str> {
        self.predicate_inside(store, predicate, 0)
    }
    /// Map roots beneath retained surrounding binders; the caller preserves their identities.
    pub(crate) fn predicate_inside(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
        depth: u16,
    ) -> Result<Predicate, &'static str> {
        self.predicate_observed::<false>(store, predicate, depth, &|_, _| Ok(()))
    }
    fn predicate_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Predicate, &'static str> {
        let ty = self.apply_inside_observed::<CHARGED>(store, predicate.ty, depth, charge)?;
        let bound = match predicate.bound {
            Requirement::Trait(id)
            | Requirement::ConstTrait(id)
            | Requirement::MaybeConstTrait(id) => {
                let (_, constness) = predicate
                    .bound
                    .trait_parts()
                    .ok_or("missing trait qualification")?;
                Requirement::trait_application(
                    self.trait_application_observed::<CHARGED>(store, id, depth, charge)?,
                    constness,
                )
            }
            Requirement::Outlives(region) => {
                Requirement::Outlives(self.region_observed::<CHARGED>(region, depth, charge)?)
            }
            Requirement::ProjectionEqual(result) => Requirement::ProjectionEqual(
                self.apply_inside_observed::<CHARGED>(store, result, depth, charge)?,
            ),
            bound => bound,
        };
        Ok(Predicate { ty, bound })
    }

    /// Repeated applications share the folder's argument environment and memoized types.
    /// # Errors
    /// Reports the same malformed data and work limits as `predicate`.
    pub fn trait_application(
        &mut self,
        store: &mut Store,
        id: ApplicationId,
    ) -> Result<ApplicationId, &'static str> {
        self.trait_application_inside(store, id, 0)
    }
    fn trait_application_inside(
        &mut self,
        store: &mut Store,
        id: ApplicationId,
        depth: u16,
    ) -> Result<ApplicationId, &'static str> {
        self.trait_application_observed::<false>(store, id, depth, &|_, _| Ok(()))
    }
    fn trait_application_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        id: ApplicationId,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<ApplicationId, &'static str> {
        if CHARGED {
            charge(1, 0)?;
        }
        let depth = self.cache_depth(depth);
        self.region_work(1)?;
        if let Some(result) = self
            .applications
            .as_ref()
            .and_then(|cache| cache.get(id, depth))
        {
            return Ok(result);
        }
        let application = store
            .trait_application(id)
            .ok_or("invalid substituted trait application")?;
        let arguments = store
            .arguments(application.arguments)
            .ok_or("invalid substituted trait arguments")?;
        let length = arguments.len();
        if CHARGED {
            let mut growth = crate::support::retained::Growth::vector::<
                crate::types::arguments::Argument,
            >(0, 0, length);
            if let Some(cache) = &self.applications {
                growth.include(cache.insertion_growth(depth));
            } else {
                growth.bytes = growth.bytes.saturating_add(size_of::<Applications>());
                growth.include(Applications::default().insertion_growth(depth));
            }
            charge(growth.work, growth.bytes)?;
        }
        self.account_arguments(length.saturating_add(1))?;
        let cache = self.applications.as_ref().map_or_else(
            || {
                size_of::<Applications>()
                    .saturating_add(Applications::default().planned_insert(depth))
            },
            |a| a.planned_insert(depth),
        );
        let scratch = self.reserve_region_scratch(
            super::bounded::growth::<crate::types::arguments::Argument>(length)
                .saturating_add(cache),
        )?;
        let mut arguments = arguments.collect::<Vec<_>>();
        let result = (|| {
            for argument in &mut arguments {
                *argument =
                    self.apply_argument_observed::<CHARGED>(store, *argument, depth, charge)?;
            }
            if CHARGED {
                charge(arguments.len().saturating_add(1), 0)?;
                let mut growth = store.argument_interning_growth(&arguments);
                growth.include(store.trait_application_growth());
                charge(growth.work.saturating_add(arguments.len()), growth.bytes)?;
            }
            let arguments = store.intern_arguments(&arguments)?;
            let result = store.intern_trait_application(Application {
                arguments,
                ..application
            })?;
            self.applications
                .get_or_insert_with(Default::default)
                .insert(id, depth, result);
            Ok(result)
        })();
        self.release_region_scratch(scratch)?;
        result
    }
}
