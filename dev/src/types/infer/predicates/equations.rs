//! Probe a complete trait goal without separating roots that share inference variables.
use super::{
    Argument, Canonical, CanonicalArgument, Context, Error, Predicate, Requirement, Store, Trait,
};
use crate::types::parameters::predicates::Bound;

impl Context {
    /// Import Self and every argument with one shared reachable-graph mapping.
    /// Failed candidate equations in this context cannot constrain the source.
    /// # Errors
    /// Rejects invalid identities, compact overflow and exhausted inference resources.
    pub fn import_predicate(
        &mut self,
        store: &Store,
        source: &mut Self,
        predicate: &Predicate,
    ) -> Result<Predicate, Error> {
        let mut mappings = super::super::import::Import::default();
        let subject = self.import_with(store, source, predicate.subject, &mut mappings)?;
        let requirement = match &predicate.requirement {
            Requirement::Builtin(bound) => Requirement::Builtin(*bound),
            Requirement::Trait(trait_) => {
                self.account_work(1 + trait_.arguments.len())?;
                let mut arguments = Vec::with_capacity(trait_.arguments.len());
                for &argument in &trait_.arguments {
                    arguments.push(match argument {
                        Argument::Lifetime(region) => Argument::Lifetime(region),
                        Argument::Type(term) => {
                            Argument::Type(self.import_with(store, source, term, &mut mappings)?)
                        }
                        Argument::Const(term) => {
                            Argument::Const(mappings.constant(store, source, self, term)?)
                        }
                    });
                }
                Requirement::Trait(Box::new(Trait {
                    item: trait_.item,
                    arguments,
                }))
            }
        };
        Ok(Predicate {
            subject,
            requirement,
        })
    }

    /// Destructively equate a goal with one canonical assumption. Selection must
    /// probe alternatives in isolated contexts before committing these equations.
    /// Region equalities are obligations, not proof that two caller lifetimes agree.
    /// # Errors
    /// Rejects mismatched traits/arguments, invalid identities, occurs cycles and work limits.
    pub fn equate_trait_predicate(
        &mut self,
        store: &Store,
        predicate: &Predicate,
        assumption: Canonical,
    ) -> Result<(), Error> {
        let (Requirement::Trait(trait_), Bound::Trait(id)) =
            (&predicate.requirement, assumption.bound)
        else {
            return Err(Error::Mismatch);
        };
        let application = store.trait_application(id).ok_or(Error::InvalidIdentity)?;
        let arguments = store
            .arguments(application.arguments)
            .ok_or(Error::InvalidIdentity)?;
        if application.item != trait_.item || arguments.len() != trait_.arguments.len() {
            return Err(Error::Mismatch);
        }
        self.account_work(1 + arguments.len())?;
        let subject = self.known(store, assumption.ty)?;
        self.equate(store, predicate.subject, subject)?;
        for (&inferred, canonical) in trait_.arguments.iter().zip(arguments) {
            match (inferred, canonical) {
                (Argument::Type(term), CanonicalArgument::Type(ty)) => {
                    let known = self.known(store, ty)?;
                    self.equate(store, term, known)?;
                }
                (Argument::Const(term), CanonicalArgument::Const(id)) => {
                    let known = self.known_const(store, id)?;
                    self.equate_consts(term, known)?;
                }
                (Argument::Lifetime(a), CanonicalArgument::Lifetime(b)) => {
                    if a != b {
                        self.region_equalities.push((a, b));
                    }
                }
                _ => return Err(Error::Mismatch),
            }
        }
        Ok(())
    }
}
