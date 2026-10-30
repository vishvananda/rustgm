//! Reservation declarations preserve checking without acquiring positive impl authority.
use super::{Checker, DefId, Error, NodeKey};
use crate::sema::traits::implementations::Polarity;
impl Checker<'_> {
    pub(super) fn reservation_impl(&self, source: Option<NodeKey>, owner: DefId) -> Result<bool, Error> {
        let registry = self.facts.generics.implementations.as_ref()
            .ok_or_else(|| self.implementation_error(source, "missing impl polarity registry"))?;
        registry.polarity(&self.facts.types, &self.facts.associated, owner)
            .map(|polarity| polarity == Polarity::Reservation)
            .map_err(|error| self.implementation_error(source, error))
    }
    pub(super) fn reservation_pair(
        &self,
        source: Option<NodeKey>,
        owners: [DefId; 2],
        steps: &mut usize,
    ) -> Result<bool, Error> {
        *steps = steps.saturating_add(2);
        if *steps > 1_000_000 {
            return Err(self.implementation_error(source, "trait implementation overlap work budget exceeded"));
        }
        for owner in owners {
            if self.reservation_impl(source, owner)? { return Ok(true); }
        }
        Ok(false)
    }
}
