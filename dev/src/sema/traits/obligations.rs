//! Selection evidence shares the typed lifetime proof representation with later body gates.
mod body;
mod instantiate;
mod restoration;
pub(crate) use crate::types::lifetime_proof::Obligations;
impl Obligations {
    pub(in crate::sema) fn append(
        &mut self,
        evidence: super::selection::Evidence,
    ) -> Result<(), &'static str> {
        let mut proof = Self::new(evidence.regions.start);
        proof.regions = evidence.regions;
        proof.equalities = evidence.equalities;
        proof.outlives = evidence.outlives;
        proof.types = evidence.type_outlives;
        self.extend(proof)
    }
}
