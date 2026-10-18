//! Predicate substitution transforms both the subject and the complete trait arguments.
use super::Bound;
use crate::types::{
    Store,
    parameters::{
        clauses::{Clause, Quantified},
        predicates::{Bound as Requirement, Predicate},
    },
    traits::{Application, ApplicationId},
};
use std::collections::HashMap;

#[derive(Default)]
pub(super) struct Applications {
    root: HashMap<ApplicationId, ApplicationId>,
    #[expect(
        clippy::box_collection,
        reason = "ordinary predicate folders never allocate a scoped application table"
    )]
    scoped: Option<Box<HashMap<(ApplicationId, u16), ApplicationId>>>,
}
impl Applications {
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
    /// Substitute enclosing arguments without opening a clause's own quantifier.
    /// Both the subject and every requirement input retain the same binder depth.
    /// Opening that quantifier is a separate scoped proof operation.
    /// # Errors
    /// Reports malformed clauses and the same argument/work failures as `predicate`.
    pub fn clause(&mut self, store: &mut Store, clause: Clause) -> Result<Clause, &'static str> {
        Ok(match clause {
            Clause::Predicate(predicate) => Clause::Predicate(self.predicate(store, predicate)?),
            Clause::ForAll(id) => {
                let value = store.quantified(id).ok_or("invalid quantified predicate")?;
                let predicate = self.predicate_inside(store, value.predicate, 1)?;
                Clause::ForAll(store.intern_quantified(Quantified { predicate, ..value })?)
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
    fn predicate_inside(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
        depth: u16,
    ) -> Result<Predicate, &'static str> {
        let ty = self.apply_inside(store, predicate.ty, depth)?;
        let bound = match predicate.bound {
            Requirement::Trait(id) => {
                Requirement::Trait(self.trait_application_inside(store, id, depth)?)
            }
            Requirement::Outlives(region) => Requirement::Outlives(self.region(region, depth)?),
            Requirement::ProjectionEqual(result) => {
                Requirement::ProjectionEqual(self.apply_inside(store, result, depth)?)
            }
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
        self.account_arguments(arguments.len().saturating_add(1))?;
        let mut arguments = arguments.collect::<Vec<_>>();
        for argument in &mut arguments {
            *argument = self.apply_argument_inside(store, *argument, depth)?;
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
    }
}
