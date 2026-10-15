//! Predicate substitution transforms both the subject and the complete trait arguments.
use super::Bound;
use crate::types::{
    Store,
    parameters::predicates::{Bound as Requirement, Predicate},
    traits::{Application, ApplicationId},
};

impl Bound {
    /// Use the same immutable parameter/lifetime mapping as signature substitution.
    /// # Errors
    /// Rejects malformed applications, missing arguments and exhausted fold work.
    pub fn predicate(
        &mut self,
        store: &mut Store,
        predicate: Predicate,
    ) -> Result<Predicate, &'static str> {
        let ty = self.apply(store, predicate.ty)?;
        let bound = match predicate.bound {
            Requirement::Trait(id) => Requirement::Trait(self.trait_application(store, id)?),
            Requirement::Outlives(region) => Requirement::Outlives(self.region(region, 0)?),
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
        if let Some(&result) = self.applications.as_ref().and_then(|cache| cache.get(&id)) {
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
            *argument = self.apply_argument(store, *argument)?;
        }
        let arguments = store.intern_arguments(&arguments)?;
        let result = store.intern_trait_application(Application {
            arguments,
            ..application
        })?;
        self.applications
            .get_or_insert_with(Default::default)
            .insert(id, result);
        Ok(result)
    }
}
