//! Owned contract vectors/proof payloads exclude immutable shared environments.
use super::Contract;
impl Contract {
    pub(crate) fn retained_bytes(&self) -> usize {
        [
            size_of::<Self>(),
            crate::support::retained::vector(&self.assumptions),
            crate::support::retained::vector(&self.parameters),
            crate::support::retained::vector(&self.type_assumptions),
            crate::support::retained::vector(&self.obligations),
            crate::support::retained::vector(&self.proofs),
            self.proofs
                .iter()
                .map(|p| {
                    crate::support::retained::vector(&p.sources)
                        .saturating_add(p.proof.allocation_bytes())
                })
                .fold(0, usize::saturating_add),
        ]
        .into_iter()
        .fold(0, usize::saturating_add)
    }
}
