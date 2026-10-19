//! Complete operation inputs share one import graph and independent equality probing.
use super::{Application, Argument, Context, Error, Store};
use crate::types::{
    GenericArgs,
    arguments::Argument as Canonical,
    infer::import::Import,
    parameters::predicates::{Bound, Predicate},
    projection,
};

impl Context {
    /// Import the complete application with one mapping across both argument groups.
    /// Shared type and const variables remain shared; trial equations stay private.
    /// # Errors
    /// Rejects invalid identities and exhausted inference resources.
    pub fn import_projection(
        &mut self,
        store: &Store,
        source: &mut Self,
        application: &Application,
    ) -> Result<Application, Error> {
        let mut mappings = Import::default();
        let predicate = application.predicate.map_arguments(|arg| {
            self.step()?;
            mappings.argument(store, source, self, arg)
        })?;
        self.account_work(application.arguments.len())?;
        let arguments = application
            .arguments
            .iter()
            .map(|&arg| mappings.argument(store, source, self, arg))
            .collect::<Result<_, _>>()?;
        Ok(Application {
            item: application.item,
            predicate,
            arguments,
        })
    }

    /// Commit equations on complete inputs after choosing a compatible operation.
    /// Result equality alone never authorizes this operation or identifies inputs.
    /// # Errors
    /// Rejects different members, kinds, arities, incompatible inputs and work limits.
    pub fn equate_projection_inputs(
        &mut self,
        store: &mut Store,
        goal: &Application,
        candidate: projection::Application,
    ) -> Result<(), Error> {
        self.equate_projection_inputs_inside(store, goal, candidate, &[])
    }

    /// Commit complete operation inputs beneath the original enclosing binders.
    /// Ambient lifetimes stay rigid; this grants no equality from result types.
    /// # Errors
    /// Rejects mismatched inputs, ambient lifetime capture and exhausted work.
    pub fn equate_projection_inputs_inside(
        &mut self,
        store: &mut Store,
        goal: &Application,
        candidate: projection::Application,
        ambient: &[u32],
    ) -> Result<(), Error> {
        if goal.item != candidate.item {
            return Err(Error::Mismatch);
        }
        self.equate_trait_predicate_inside(
            store,
            &goal.predicate,
            Predicate {
                ty: candidate.subject,
                bound: Bound::Trait(candidate.trait_application),
            },
            ambient,
        )?;
        self.equate_canonical_inputs(store, None, &goal.arguments, candidate.arguments, ambient)
    }

    /// Resolve one argument group with a shared reachable-type traversal.
    /// Unknown inputs remain unknown; this does not perform numeric fallback.
    /// # Errors
    /// Rejects unresolved inputs, invalid identities and exhausted inference work.
    pub fn resolved_arguments(
        &mut self,
        store: &mut Store,
        arguments: &[Argument],
    ) -> Result<GenericArgs, Error> {
        let mut resolution = super::super::resolve::Resolution::default();
        self.account_work(arguments.len())?;
        let values = arguments
            .iter()
            .map(|&argument| {
                Ok(match argument {
                    Argument::Lifetime(region) => Canonical::Lifetime(region),
                    Argument::Type(term) => {
                        Canonical::Type(self.resolve_with(store, term, &mut resolution)?)
                    }
                    Argument::Const(term) => Canonical::Const(self.resolved_const(store, term)?),
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;
        store
            .intern_arguments(&values)
            .map_err(|_| Error::InvalidIdentity)
    }

    /// Reuse canonical arguments without introducing parameter inference variables.
    /// Lifetime opening, formation and parameter substitution belong to the caller.
    /// # Errors
    /// Rejects malformed canonical arguments and exhausted inference resources.
    pub fn known_arguments(
        &mut self,
        store: &Store,
        arguments: GenericArgs,
    ) -> Result<Vec<Argument>, Error> {
        let arguments = store.arguments(arguments).ok_or(Error::InvalidIdentity)?;
        self.account_work(arguments.len())?;
        arguments
            .map(|arg| {
                Ok(match arg {
                    Canonical::Lifetime(region) => Argument::Lifetime(region),
                    Canonical::Type(ty) => Argument::Type(self.known(store, ty)?),
                    Canonical::Const(id) => Argument::Const(self.known_const(store, id)?),
                })
            })
            .collect()
    }
}
