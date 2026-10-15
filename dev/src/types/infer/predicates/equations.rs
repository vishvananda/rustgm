//! Probe a complete trait goal without separating roots that share inference variables.
use super::{
    Argument, Canonical, CanonicalArgument, Context, Error, Predicate, Requirement, Store, Trait,
};
use crate::types::parameters::predicates::Bound;

impl Context {
    /// Existential header compatibility, with lifetime proof deliberately deferred.
    /// This overapproximation may establish disjointness, never body permissions.
    /// # Errors
    /// Rejects incompatible traits/arguments, malformed data and exhausted inference work.
    pub fn equate_predicate_shapes(
        &mut self,
        store: &Store,
        left: &Predicate,
        right: &Predicate,
    ) -> Result<(), Error> {
        let (Requirement::Trait(a), Requirement::Trait(b)) =
            (&left.requirement, &right.requirement)
        else {
            return Err(Error::InvalidIdentity);
        };
        if a.item != b.item || a.arguments.len() != b.arguments.len() {
            return Err(Error::Mismatch);
        }
        self.account_work(1 + a.arguments.len())?;
        self.equate_shape(store, left.subject, right.subject)?;
        for (&a, &b) in a.arguments.iter().zip(&b.arguments) {
            match (a, b) {
                (Argument::Type(a), Argument::Type(b)) => self.equate_shape(store, a, b)?,
                (Argument::Const(a), Argument::Const(b)) => self.equate_consts(a, b)?,
                (Argument::Lifetime(_), Argument::Lifetime(_)) => {}
                _ => return Err(Error::Mismatch),
            }
        }
        Ok(())
    }
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
        self.equate_scoped(store, predicate.subject, assumption.ty)?;
        for (&inferred, canonical) in trait_.arguments.iter().zip(arguments) {
            match (inferred, canonical) {
                (Argument::Type(term), CanonicalArgument::Type(ty)) => {
                    self.equate_scoped(store, term, ty)?;
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
