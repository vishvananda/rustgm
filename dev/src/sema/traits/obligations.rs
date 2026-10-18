//! Selection evidence shares the typed lifetime proof representation with later body gates.
mod body;
mod instantiate;
mod restoration;
pub(crate) use crate::types::lifetime_proof::Obligations;
pub(in crate::sema) use body::Reservation;
impl Obligations {
    pub(in crate::sema) fn append(
        &mut self,
        evidence: super::selection::Evidence,
    ) -> Result<(), &'static str> {
        self.extend(evidence.proof)
    }
}
