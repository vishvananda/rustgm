//! Trait obligations retain inference arguments until the whole body is solved.
mod equations;
use super::{Context, Error, Instantiation, Solved, Store, TermId, TypeId, arguments::Argument};
use crate::types::{
    ItemId,
    arguments::Argument as CanonicalArgument,
    parameters::predicates::{Bound, Predicate as Canonical},
    traits::Application,
};

enum Requirement {
    Builtin(Bound),
    Trait(Box<Trait>),
}
struct Trait {
    item: ItemId,
    arguments: Vec<Argument>,
}
pub struct Predicate {
    subject: TermId,
    requirement: Requirement,
}
impl Predicate {
    #[must_use]
    pub const fn builtin_bound(&self) -> Option<Bound> {
        match &self.requirement {
            Requirement::Builtin(bound) => Some(*bound),
            Requirement::Trait(_) => None,
        }
    }
    #[must_use]
    pub const fn subject(&self) -> TermId {
        self.subject
    }
    #[must_use]
    pub fn trait_arguments(&self) -> Option<(ItemId, &[Argument])> {
        match &self.requirement {
            Requirement::Trait(trait_) => Some((trait_.item, &trait_.arguments)),
            Requirement::Builtin(_) => None,
        }
    }
}
impl Context {
    /// Share parameter identities/equivalence classes across Self and trait arguments.
    /// Lifetime substitution precedes this operation, as for ordinary type templates.
    /// # Errors
    /// Rejects malformed predicates, kind mismatches and exhausted inference work.
    pub fn instantiate_predicate(
        &mut self,
        store: &mut Store,
        predicate: Canonical,
        substitution: &mut Instantiation,
    ) -> Result<Predicate, Error> {
        let subject = self.instantiate_with(store, predicate.ty, substitution)?;
        let requirement = if let Bound::Trait(id) = predicate.bound {
            let application = store.trait_application(id).ok_or(Error::InvalidIdentity)?;
            let arguments = store
                .arguments(application.arguments)
                .ok_or(Error::InvalidIdentity)?;
            self.account_work(arguments.len())?;
            let arguments = arguments.collect::<Vec<_>>();
            let mut inferred = Vec::with_capacity(arguments.len());
            for argument in arguments {
                inferred.push(match argument {
                    CanonicalArgument::Lifetime(region) => Argument::Lifetime(region),
                    CanonicalArgument::Type(ty) => {
                        Argument::Type(self.instantiate_with(store, ty, substitution)?)
                    }
                    CanonicalArgument::Const(id) => {
                        Argument::Const(self.instantiate_const(store, id, substitution)?)
                    }
                });
            }
            Requirement::Trait(Box::new(Trait {
                item: application.item,
                arguments: inferred,
            }))
        } else {
            Requirement::Builtin(predicate.bound)
        };
        Ok(Predicate {
            subject,
            requirement,
        })
    }

    /// Resolve only already constrained data; numeric fallback belongs to body completion.
    /// # Errors
    /// Reports unresolved arguments, malformed identities and exhausted work.
    pub fn resolved_predicate(
        &mut self,
        store: &mut Store,
        predicate: &Predicate,
    ) -> Result<Canonical, Error> {
        self.predicate_with(store, predicate, Self::resolved)
    }

    /// Finish both types and retained obligations before discarding inference state.
    /// Const arguments that occur only in a trait predicate still require a solution.
    /// # Errors
    /// Reports the failures of `finish` and unresolved or invalid predicate arguments.
    pub fn finish_with_predicates<'p>(
        mut self,
        store: &mut Store,
        predicates: impl Iterator<Item = &'p Predicate>,
    ) -> Result<(Solved, Vec<Canonical>), Error> {
        let mut solved = self.solve_types(store)?;
        let mut completed = Vec::new();
        for predicate in predicates {
            self.step()?;
            completed.push(self.predicate_with(store, predicate, |_, _, term| {
                solved
                    .types
                    .get(term.0 as usize)
                    .copied()
                    .ok_or(Error::InvalidIdentity)
            })?);
        }
        solved.steps = self.steps;
        Ok((solved, completed))
    }

    fn predicate_with(
        &mut self,
        store: &mut Store,
        predicate: &Predicate,
        mut type_of: impl FnMut(&mut Self, &mut Store, TermId) -> Result<TypeId, Error>,
    ) -> Result<Canonical, Error> {
        let ty = type_of(self, store, predicate.subject)?;
        let bound = match &predicate.requirement {
            Requirement::Builtin(bound) => *bound,
            Requirement::Trait(trait_) => {
                self.account_work(trait_.arguments.len())?;
                let mut arguments = Vec::with_capacity(trait_.arguments.len());
                for &argument in &trait_.arguments {
                    arguments.push(match argument {
                        Argument::Lifetime(region) => CanonicalArgument::Lifetime(region),
                        Argument::Type(term) => {
                            CanonicalArgument::Type(type_of(self, store, term)?)
                        }
                        Argument::Const(term) => {
                            CanonicalArgument::Const(self.resolved_const(store, term)?)
                        }
                    });
                }
                let arguments = store
                    .intern_arguments(&arguments)
                    .map_err(|_| Error::InvalidIdentity)?;
                Bound::Trait(
                    store
                        .intern_trait_application(Application {
                            item: trait_.item,
                            arguments,
                        })
                        .map_err(|_| Error::InvalidIdentity)?,
                )
            }
        };
        Ok(Canonical { ty, bound })
    }
}
