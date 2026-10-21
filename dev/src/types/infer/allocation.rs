//! Retained inference owners report their collections without resolving any terms.
use super::Context;
use crate::support::retained::Allocation;
impl Context {
    pub(crate) fn retained_allocation(&self) -> (usize, usize) {
        let mut total = Allocation::default();
        total.vector(&self.terms);
        total.vector(&self.edges);
        total.map(&self.known);
        total.vector(&self.pairs);
        total.set(&self.seen);
        total.vector(&self.occurs_work);
        total.vector(&self.child_scratch);
        total.vector(&self.marks);
        total.vector(&self.region_equalities);
        total.include(self.generalization.allocation());
        if let Some(value) = &self.defining {
            total.boxed(value.as_ref());
            total.include(value.allocation());
        }
        if let Some(value) = &self.signature_slots {
            total.boxed(value.as_ref());
            total.include(value.allocation());
        }
        if let Some(value) = &self.arguments {
            total.boxed(value.as_ref());
            total.include(value.allocation());
        }
        if let Some(value) = &self.dependencies {
            total.boxed(value.as_ref());
            total.include(value.allocation());
        }
        if let Some(value) = &self.resolution {
            total.boxed(value.as_ref());
            total.include(value.allocation());
        }
        if let Some(value) = &self.settled {
            total.boxed(value.as_ref());
            total.map(value);
        }
        (total.bytes, total.visits)
    }
}
