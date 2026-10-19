//! Open a whole inferred predicate through one shared binder mapping.
use super::{Argument, Context, Error, Predicate, Store};
use crate::types::substitute::Bound;

impl Context {
    /// Keep unknown type/const identities while replacing a predicate's binder.
    /// Shared input prefixes are traversed once, including an equality's result.
    pub(crate) fn map_predicate_regions(
        &mut self,
        store: &mut Store,
        predicate: &Predicate,
        bound: &mut Bound,
    ) -> Result<Predicate, Error> {
        let roots: Vec<_> = predicate
            .arguments()
            .filter_map(|argument| match argument {
                Argument::Type(root) => Some(root),
                Argument::Lifetime(_) | Argument::Const(_) => None,
            })
            .collect();
        let mut mapped = self.map_region_roots(store, &roots, bound)?.into_iter();
        predicate.map_arguments(|argument| {
            self.step()?;
            Ok(match argument {
                Argument::Type(_) => Argument::Type(mapped.next().ok_or(Error::InvalidIdentity)?),
                Argument::Lifetime(region) => {
                    let before = bound.steps;
                    let region = bound
                        .region(region, 0)
                        .map_err(|_| Error::InvalidIdentity)?;
                    self.account_work(bound.steps.saturating_sub(before))?;
                    Argument::Lifetime(region)
                }
                Argument::Const(_) => argument,
            })
        })
    }
}
