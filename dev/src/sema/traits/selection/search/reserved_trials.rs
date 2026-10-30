//! A compatible reservation is ambiguity, never a completed implementation node.
use super::{Declarations, Search, Trial};
use crate::sema::traits::implementations::{Polarity, Purpose};
impl Search<'_> {
    pub(super) fn deferred_reservation(
        &self,
        facts: &Declarations,
        trial: &Trial,
    ) -> Result<bool, String> {
        if facts
            .generics
            .implementations
            .as_ref()
            .ok_or("reservation trial lost its actual registry")?
            .polarity(&facts.types, &facts.associated, trial.instance.owner)?
            != Polarity::Reservation
        {
            return Ok(false);
        }
        if self.query.purpose != Purpose::Coherence {
            return Err("reservation cannot complete ordinary trait selection".into());
        }
        // The caller has checked ordinary impl conditions. This private query
        // can retain possibility, never selected members or substitution proof.
        Ok(true)
    }
}
