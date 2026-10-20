//! Concrete erasure creates obligations in the object's complete trait binder.
use super::{Constructor, Context, Error, Store, TermId};
use crate::types::{
    infer::{arguments::Argument, predicates::Predicate},
    parameters::predicates::Bound,
    substitute,
};

/// The subject and every trait/equality input share this optional quantifier.
pub struct Requirement {
    pub predicate: Predicate,
    pub bound_regions: Option<u32>,
}

impl Context {
    /// Retain type inference holes while forming concrete-to-object proof goals.
    /// The outer object lifetime is deliberately outside the trait quantifier.
    /// This creates obligations, not evidence that conversion is permitted.
    /// # Errors
    /// Rejects malformed object inputs, invalid projections and exhausted work.
    pub fn object_requirements(
        &mut self,
        store: &mut Store,
        concrete: TermId,
        target: TermId,
    ) -> Result<Vec<Requirement>, Error> {
        self.object_constraints(store, concrete, target, true)
    }
    /// Form an object upcast's requirements without requiring its unsized Self
    /// to be Sized. Trait evidence must separately come from intrinsic bounds.
    /// # Errors
    /// Rejects a non-object source and the same malformed inputs as object conversion.
    pub fn upcast_requirements(
        &mut self,
        store: &mut Store,
        source: TermId,
        target: TermId,
    ) -> Result<Vec<Requirement>, Error> {
        if !matches!(
            self.materialize(store, source)?,
            Constructor::Dynamic { .. }
        ) {
            return Err(Error::InvalidIdentity);
        }
        self.object_constraints(store, source, target, false)
    }
    fn object_constraints(
        &mut self,
        store: &mut Store,
        concrete: TermId,
        target: TermId,
        sized: bool,
    ) -> Result<Vec<Requirement>, Error> {
        let target = self.root(target)?;
        let Constructor::Dynamic {
            shape,
            arguments,
            lifetimes,
        } = self.materialize(store, target)?
        else {
            return Err(Error::InvalidIdentity);
        };
        let lifetimes = self.object_lifetimes(lifetimes)?;
        let value = store.dynamic_shape(shape).ok_or(Error::InvalidIdentity)?;
        let mut arguments = self.application_arguments(target, arguments, 0)?;
        let mut output = Vec::new();
        if sized {
            output.push(Requirement {
                predicate: Predicate::builtin(concrete, Bound::Sized),
                bound_regions: None,
            });
        }
        output.push(Requirement {
            predicate: Predicate::builtin(concrete, Bound::Outlives(lifetimes.region)),
            bound_regions: None,
        });
        let subject = if lifetimes.bound_regions == 0 {
            self.open_object_arguments(store, &mut arguments)?;
            concrete
        } else {
            self.shift_regions(store, concrete, 1)?.term
        };
        let bound_regions = (lifetimes.bound_regions != 0).then_some(lifetimes.bound_regions);
        let mut start = value.principal_arguments as usize;
        if let Some(item) = value.principal {
            let inputs = arguments
                .get(..start)
                .ok_or(Error::InvalidIdentity)?
                .to_vec();
            output.push(Requirement {
                predicate: Predicate::trait_application(subject, item, inputs),
                bound_regions,
            });
        }
        for binding in store
            .dynamic_bindings(shape)
            .ok_or(Error::InvalidIdentity)?
            .to_vec()
        {
            let end = start
                .checked_add(binding.arguments as usize)
                .ok_or(Error::Overflow)?;
            let inputs = arguments.get(start..end).ok_or(Error::InvalidIdentity)?;
            let Some(&Argument::Type(result)) = arguments.get(end) else {
                return Err(Error::InvalidIdentity);
            };
            let projected = self.projected(store, binding.item, subject, inputs, &[])?;
            output.push(Requirement {
                predicate: Predicate::equality(projected, result),
                bound_regions,
            });
            start = end + 1;
        }
        if start != arguments.len() {
            return Err(Error::InvalidIdentity);
        }
        for marker in store
            .dynamic_auto_traits(shape)
            .ok_or(Error::InvalidIdentity)?
            .to_vec()
        {
            output.push(Requirement {
                predicate: Predicate::builtin(subject, Bound::Auto(marker)),
                bound_regions,
            });
        }
        Ok(output)
    }

    // Share the binder fold across all object inputs, including repeated equality roots.
    fn open_object_arguments(
        &mut self,
        store: &mut Store,
        arguments: &mut [Argument],
    ) -> Result<(), Error> {
        let roots = arguments
            .iter()
            .filter_map(|argument| match argument {
                Argument::Type(ty) => Some(*ty),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut lower = substitute::Bound::lower(1);
        let mapped = self.map_region_roots(store, &roots, &mut lower)?;
        let mut mapped = mapped.into_iter();
        for argument in arguments {
            match argument {
                Argument::Lifetime(region) => {
                    *region = lower
                        .region(*region, 0)
                        .map_err(|_| Error::InvalidIdentity)?;
                }
                Argument::Type(ty) => *ty = mapped.next().ok_or(Error::InvalidIdentity)?,
                Argument::Const(_) => {}
            }
        }
        Ok(())
    }
}
