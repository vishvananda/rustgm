//! Logical retained proof payload excludes immutable shared binder tables.
use super::Obligations;
impl Obligations {
    pub(crate) fn allocation_bytes(&self) -> usize {
        let mut work = vec![self];
        let mut bytes = 0usize;
        while let Some(proof) = work.pop() {
            bytes = bytes
                .saturating_add(crate::support::retained::vector(&proof.equalities))
                .saturating_add(crate::support::retained::vector(&proof.outlives))
                .saturating_add(crate::support::retained::vector(&proof.types));
            if let Some(q) = &proof.quantified {
                bytes = bytes
                    .saturating_add(size_of_val(q.as_ref()))
                    .saturating_add(crate::support::retained::vector(&q.blocks))
                    .saturating_add(crate::support::retained::vector(&q.gaps))
                    .saturating_add(q.origins.as_ref().map_or(0, |o| o.allocation_bytes()));
                work.extend(q.blocks.iter().map(|block| &block.proof));
            }
        }
        bytes
    }
}
