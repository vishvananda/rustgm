//! Logical retained proof payload excludes immutable shared binder tables.
use super::Obligations;
use crate::support::retained::Allocation;
impl Obligations {
    pub(crate) fn shallow_allocation(&self) -> Allocation {
        let mut total = Allocation::default();
        total.vector(&self.equalities);
        total.vector(&self.outlives);
        total.vector(&self.types);
        if let Some(q) = &self.quantified {
            total.boxed(q.as_ref());
            total.vector(&q.blocks);
            total.vector(&q.gaps);
            if let Some(origins) = &q.origins {
                total.add(origins.allocation_bytes());
            }
        }
        total
    }
    pub(crate) fn retained_allocation(&self) -> Allocation {
        let mut total = Allocation::default();
        let mut work = vec![self];
        while let Some(proof) = work.pop() {
            total.include(proof.shallow_allocation());
            if let Some(q) = &proof.quantified {
                work.extend(q.blocks.iter().map(|block| &block.proof));
            }
        }
        total
    }
    pub(crate) fn allocation_bytes(&self) -> usize {
        self.retained_allocation().bytes
    }
}
