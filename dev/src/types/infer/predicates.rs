//! Trait obligations retain inference arguments until the whole body is solved.
mod binder_mapping;
mod candidate_headers;
mod equations;
mod normal_premises;
#[cfg(test)]
#[path = "../../../tests/invariants/qualifiers.rs"]
mod source_effect_tests;
use super::{Context, Error, Instantiation, Solved, Store, TermId, TypeId, arguments::Argument};
use crate::support::constness::Constness;
use crate::types::{
    ItemId,
    arguments::Argument as CanonicalArgument,
    parameters::predicates::{Bound, Predicate as Canonical},
    traits::Application,
};

enum Requirement {
    Builtin(Bound),
    EmptyTrait {
        item: ItemId,
        constness: Constness,
    },
    SingleTrait {
        item: ItemId,
        constness: Constness,
        argument: Argument,
    },
    Trait(Box<Trait>),
    ProjectionEqual(TermId),
}
struct Trait {
    constness: Constness,
    item: ItemId,
    arguments: Vec<Argument>,
}
impl Requirement {
    fn trait_application(item: ItemId, constness: Constness, arguments: Vec<Argument>) -> Self {
        match arguments.as_slice() {
            [] => Self::EmptyTrait { item, constness },
            [argument] => Self::SingleTrait {
                item,
                constness,
                argument: *argument,
            },
            _ => Self::Trait(Box::new(Trait {
                constness,
                item,
                arguments,
            })),
        }
    }
}
pub struct Predicate {
    subject: TermId,
    requirement: Requirement,
}
#[cfg(test)]
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<Predicate>() == 24);

impl Predicate {
    pub(crate) const fn builtin(subject: TermId, bound: Bound) -> Self {
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
            Requirement::EmptyTrait { item, constness } => Requirement::EmptyTrait {
                item: *item,
                constness: *constness,
            },
            Requirement::SingleTrait {
                item,
                constness,
                argument,
            } => Requirement::SingleTrait {
                item: *item,
                constness: *constness,
                argument: map(*argument)?,
            },
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
        let mut map_argument = |argument| {
            Ok(if let Argument::Type(ty) = argument {
                Argument::Type(map(ty)?)
            } else {
                argument
            })
        };
        let requirement = match &self.requirement {
            Requirement::Builtin(bound) => Requirement::Builtin(*bound),
            Requirement::ProjectionEqual(_) => return Err(Error::InvalidIdentity),
            Requirement::EmptyTrait { item, constness } => Requirement::EmptyTrait {
                item: *item,
                constness: *constness,
            },
            Requirement::SingleTrait {
                item,
                constness,
                argument,
            } => Requirement::SingleTrait {
                item: *item,
                constness: *constness,
                argument: map_argument(*argument)?,
            },
            Requirement::Trait(trait_) => {
                let arguments = trait_
                    .arguments
                    .iter()
                    .copied()
                    .map(map_argument)
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
        Self::trait_application_with_effect(subject, item, arguments, Constness::Ordinary)
    }
    /// An operation requirement's use effect, never implementation permission.
    pub(crate) fn trait_application_with_effect(
        subject: TermId,
        item: ItemId,
        arguments: Vec<Argument>,
        constness: Constness,
    ) -> Self {
        Self {
            subject,
            requirement: Requirement::trait_application(item, constness, arguments),
        }
    }
    #[must_use]
    pub const fn builtin_bound(&self) -> Option<Bound> {
        match &self.requirement {
            Requirement::Builtin(bound) => Some(*bound),
            Requirement::Trait(_)
            | Requirement::EmptyTrait { .. }
            | Requirement::SingleTrait { .. }
            | Requirement::ProjectionEqual(_) => None,
        }
    }
    #[must_use]
    pub fn constness(&self) -> Option<Constness> {
        match &self.requirement {
            Requirement::EmptyTrait { constness, .. }
            | Requirement::SingleTrait { constness, .. } => Some(*constness),
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
        self.trait_parts()
            .map(|(item, _, arguments)| (item, arguments))
    }
    fn trait_parts(&self) -> Option<(ItemId, Constness, &[Argument])> {
        match &self.requirement {
            Requirement::EmptyTrait { item, constness } => Some((*item, *constness, &[])),
            Requirement::SingleTrait {
                item,
                constness,
                argument,
            } => Some((*item, *constness, std::slice::from_ref(argument))),
            Requirement::Trait(trait_) => Some((trait_.item, trait_.constness, &trait_.arguments)),
            Requirement::Builtin(_) | Requirement::ProjectionEqual(_) => None,
        }
    }
    /// The result shares this predicate's parameter and const inference graph.
    #[must_use]
    pub const fn projection_result(&self) -> Option<TermId> {
        match self.requirement {
            Requirement::ProjectionEqual(result) => Some(result),
            Requirement::Builtin(_)
            | Requirement::Trait(_)
            | Requirement::EmptyTrait { .. }
            | Requirement::SingleTrait { .. } => None,
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
        self.instantiate_predicate_inside(store, predicate, substitution, 0)
    }

    /// Move inserted caller terms past the predicate's surrounding binders.
    /// Lifetimes already recorded in the predicate retain their declared depth.
    pub(crate) fn instantiate_predicate_inside(
        &mut self,
        store: &mut Store,
        predicate: Canonical,
        substitution: &mut Instantiation,
        depth: u16,
    ) -> Result<Predicate, Error> {
        let subject = self.instantiate_with_depth(store, predicate.ty, substitution, depth)?;
        let requirement = if let Some((id, constness)) = predicate.bound.trait_parts() {
            let application = store.trait_application(id).ok_or(Error::InvalidIdentity)?;
            let count = store
                .arguments(application.arguments)
                .ok_or(Error::InvalidIdentity)?
                .len();
            self.account_work(count)?;
            // The canonical span stays valid as instantiation adds types to Store.
            let mut instantiate_argument =
                |index| {
                    let argument = store
                        .arguments(application.arguments)
                        .and_then(|mut arguments| arguments.nth(index))
                        .ok_or(Error::InvalidIdentity)?;
                    Ok::<_, Error>(match argument {
                        CanonicalArgument::Lifetime(region) => Argument::Lifetime(region),
                        CanonicalArgument::Type(ty) => Argument::Type(
                            self.instantiate_with_depth(store, ty, substitution, depth)?,
                        ),
                        CanonicalArgument::Const(id) => {
                            Argument::Const(self.instantiate_const(store, id, substitution)?)
                        }
                    })
                };
            if count == 1 {
                Requirement::SingleTrait {
                    item: application.item,
                    constness,
                    argument: instantiate_argument(0)?,
                }
            } else {
                let mut inferred = Vec::with_capacity(count);
                for index in 0..count {
                    inferred.push(instantiate_argument(index)?);
                }
                Requirement::trait_application(application.item, constness, inferred)
            }
        } else if let Bound::ProjectionEqual(result) = predicate.bound {
            Requirement::ProjectionEqual(self.instantiate_with_depth(
                store,
                result,
                substitution,
                depth,
            )?)
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
            Requirement::Trait(_)
            | Requirement::EmptyTrait { .. }
            | Requirement::SingleTrait { .. } => {
                let (item, constness, inferred) =
                    predicate.trait_parts().ok_or(Error::InvalidIdentity)?;
                self.account_work(inferred.len())?;
                let mut arguments = Vec::with_capacity(inferred.len());
                for &argument in inferred {
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
                        .intern_trait_application(Application { item, arguments })
                        .map_err(|_| Error::InvalidIdentity)?,
                    constness,
                )
            }
        };
        Ok(Canonical { ty, bound })
    }
}
