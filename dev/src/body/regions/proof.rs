//! Body-owned proof locations do not create semantic AST facts for imports.
use super::ProofSite;

#[derive(Clone)]
pub struct Proof {
    pub source: ProofSite,
    pub sources: Vec<ProofSite>,
    pub proof: crate::types::lifetime_proof::Obligations,
}
impl Proof {
    #[must_use]
    pub fn new(
        source: impl Into<ProofSite>,
        proof: crate::types::lifetime_proof::Obligations,
    ) -> Self {
        Self {
            source: source.into(),
            sources: Vec::new(),
            proof,
        }
    }
    pub(super) fn from_source(source: &crate::sema::model::LifetimeProof) -> Self {
        Self {
            source: source.source.into(),
            sources: source.sources.iter().copied().map(Into::into).collect(),
            proof: source.proof.clone(),
        }
    }
}
