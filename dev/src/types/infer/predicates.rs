//! Trait obligations retain inference arguments until the whole body is solved.
mod binder_mapping;
mod equations;
#[cfg(test)]
#[path = "../../../tests/invariants/qualifiers.rs"]
mod source_effect_tests;
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
    ProjectionEqual(TermId),
}
struct Trait {
    constness: crate::support::constness::Constness,
    item: ItemId,
    arguments: Vec<Argument>,
}
pub struct Predicate {
    subject: TermId,
    requirement: Requirement,
}
impl Predicate {
    pub(super) const fn builtin(subject: TermId, bound: Bound) -> Self {
        Self {
            subject,
            requirement: Requirement::Builtin(bound),
        }
    }
    pub(super) const fn equality(subject: TermId, result: TermId) -> Self {
        Self {
            subject,
            requirement: Requirement::ProjectionEqual(result),
        }
    }
    /// Complete mixed inputs, including equality results and outlives lifetimes.
    /// The order is shared with `Self::map_arguments`.
    pub fn arguments(&self) -> impl Iterator<Item = Argument> + '_ {
        std::iter::once(Argument::Type(self.subject))
            .chain(
                self.trait_arguments()
                    .into_iter()
                    .flat_map(|(_, args)| args.iter().copied()),
            )
            .chain(self.projection_result().map(Argument::Type))
            .chain(match self.builtin_bound() {
                Some(Bound::Outlives(region)) => Some(Argument::Lifetime(region)),
                _ => None,
            })
    }

    /// Map all inputs through one shared transport, retaining the requirement kind.
    /// # Errors
    /// Propagates mapping failures and rejects changed argument kinds.
    pub fn map_arguments(
        &self,
        mut map: impl FnMut(Argument) -> Result<Argument, Error>,
    ) -> Result<Self, Error> {
        let mut map = |input| {
            let output = map(input)?;
            if std::mem::discriminant(&input) != std::mem::discriminant(&output) {
                return Err(Error::InvalidIdentity);
            }
            Ok(output)
        };
        let Argument::Type(subject) = map(Argument::Type(self.subject))? else {
            return Err(Error::InvalidIdentity);
        };
        let requirement = match &self.requirement {
            Requirement::Trait(trait_) => Requirement::Trait(Box::new(Trait {
                constness: trait_.constness,
                item: trait_.item,
                arguments: trait_
                    .arguments
                    .iter()
                    .copied()
                    .map(&mut map)
                    .collect::<Result<_, _>>()?,
            })),
            Requirement::ProjectionEqual(result) => {
                let Argument::Type(result) = map(Argument::Type(*result))? else {
                    return Err(Error::InvalidIdentity);
                };
                Requirement::ProjectionEqual(result)
            }
            Requirement::Builtin(Bound::Outlives(region)) => {
                let Argument::Lifetime(region) = map(Argument::Lifetime(*region))? else {
                    return Err(Error::InvalidIdentity);
                };
                Requirement::Builtin(Bound::Outlives(region))
            }
            Requirement::Builtin(bound) => Requirement::Builtin(*bound),
        };
        Ok(Self {
            subject,
            requirement,
        })
    }
    /// Each input root, preserving the trait's mixed argument ordering.
    pub fn input_roots(&self) -> impl Iterator<Item = TermId> + '_ {
        std::iter::once(self.subject).chain(
            self.trait_arguments()
                .into_iter()
                .flat_map(|(_, arguments)| arguments.iter())
                .filter_map(|argument| {
                    if let Argument::Type(ty) = argument {
                        Some(*ty)
                    } else {
                        None
                    }
                }),
        )
    }
    /// Construct a new input view while preserving every non-type argument.
    /// # Errors
    /// Propagates the input transformation's errors and rejects equality goals.
    pub fn map_inputs(
        &self,
        mut map: impl FnMut(TermId) -> Result<TermId, Error>,
    ) -> Result<Self, Error> {
        let subject = map(self.subject)?;
        let requirement = match &self.requirement {
            Requirement::Builtin(bound) => Requirement::Builtin(*bound),
            Requirement::ProjectionEqual(_) => return Err(Error::InvalidIdentity),
            Requirement::Trait(trait_) => {
                let arguments = trait_
                    .arguments
                    .iter()
                    .map(|&argument| {
                        Ok(if let Argument::Type(ty) = argument {
                            Argument::Type(map(ty)?)
                        } else {
                            argument
                        })
                    })
                    .collect::<Result<_, Error>>()?;
                Requirement::Trait(Box::new(Trait {
                    constness: trait_.constness,
                    item: trait_.item,
                    arguments,
                }))
            }
        };
        Ok(Self {
            subject,
            requirement,
        })
    }
    pub(crate) fn trait_application(
        subject: TermId,
        item: ItemId,
        arguments: Vec<Argument>,
    ) -> Self {
        Self::trait_application_with_effect(
            subject,
            item,
            arguments,
            crate::support::constness::Constness::Ordinary,
        )
    }
    /// An operation requirement's use effect, never implementation permission.
    pub(crate) fn trait_application_with_effect(
        subject: TermId,
        item: ItemId,
        arguments: Vec<Argument>,
        constness: crate::support::constness::Constness,
    ) -> Self {
        Self {
            subject,
            requirement: Requirement::Trait(Box::new(Trait {
                constness,
                item,
                arguments,
            })),
        }
    }
    #[must_use]
    pub const fn builtin_bound(&self) -> Option<Bound> {
        match &self.requirement {
            Requirement::Builtin(bound) => Some(*bound),
            Requirement::Trait(_) | Requirement::ProjectionEqual(_) => None,
        }
    }
    #[must_use]
    pub fn constness(&self) -> Option<crate::support::constness::Constness> {
        match &self.requirement {
            Requirement::Trait(trait_) => Some(trait_.constness),
            _ => None,
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
            Requirement::Builtin(_) | Requirement::ProjectionEqual(_) => None,
        }
    }
    /// The result shares this predicate's parameter and const inference graph.
    #[must_use]
    pub const fn projection_result(&self) -> Option<TermId> {
        match self.requirement {
            Requirement::ProjectionEqual(result) => Some(result),
            Requirement::Builtin(_) | Requirement::Trait(_) => None,
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
        let requirement = if let Some((id, constness)) = predicate.bound.trait_parts() {
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
                constness,
                item: application.item,
                arguments: inferred,
            }))
        } else if let Bound::ProjectionEqual(result) = predicate.bound {
            Requirement::ProjectionEqual(self.instantiate_with(store, result, substitution)?)
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
        let mut resolution = super::resolve::Resolution::default();
        self.predicate_with(store, predicate, |context, store, root| {
            context.resolve_with(store, root, &mut resolution)
        })
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
        let (solved, predicates, _) =
            self.finish_with_arguments(store, predicates, std::iter::empty())?;
        Ok((solved, predicates))
    }

    /// Finish independent item argument lists without inventing a value type.
    /// # Errors
    /// Rejects unresolved types, constants, obligations and invalid item arguments.
    /// The finished context retains only storage; see [`Context::recycled`].
    pub fn finish_with_arguments<'p>(
        &mut self,
        store: &mut Store,
        predicates: impl Iterator<Item = &'p Predicate>,
        arguments: impl Iterator<Item = &'p [Argument]>,
    ) -> Result<(Solved, Vec<Canonical>, Vec<crate::types::GenericArgs>), Error> {
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
        let mut items = Vec::new();
        for arguments in arguments {
            self.account_work(arguments.len())?;
            let mut values = Vec::with_capacity(arguments.len());
            for &argument in arguments {
                values.push(match argument {
                    Argument::Lifetime(region) => CanonicalArgument::Lifetime(region),
                    Argument::Type(term) => CanonicalArgument::Type(
                        *solved
                            .types
                            .get(term.0 as usize)
                            .ok_or(Error::InvalidIdentity)?,
                    ),
                    Argument::Const(term) => {
                        CanonicalArgument::Const(self.resolved_const(store, term)?)
                    }
                });
            }
            items.push(
                store
                    .intern_arguments(&values)
                    .map_err(|_| Error::InvalidIdentity)?,
            );
        }
        solved.steps = self.steps;
        Ok((solved, completed, items))
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
            Requirement::ProjectionEqual(result) => {
                Bound::ProjectionEqual(type_of(self, store, *result)?)
            }
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
                Bound::trait_application(
                    store
                        .intern_trait_application(Application {
                            item: trait_.item,
                            arguments,
                        })
                        .map_err(|_| Error::InvalidIdentity)?,
                    trait_.constness,
                )
            }
        };
        Ok(Canonical { ty, bound })
    }
}
