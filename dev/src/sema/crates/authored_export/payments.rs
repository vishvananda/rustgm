//! Exclusive exporter queries retain attempted costs on success and failure.
use super::Stage;
use crate::support::{crates, retained::Payment};
use std::cell::Cell;
#[cfg(test)]
#[path = "../../../../tests/invariants/publication_budget.rs"]
mod tests;

impl Stage {
    pub(crate) fn paying<R>(
        &mut self,
        query: impl FnOnce(Payment<'_>) -> Result<R, &'static str>,
    ) -> Result<R, &'static str> {
        let steps = Cell::new(self.steps);
        let bytes = Cell::new(self.bytes);
        let scratch = self.scratch;
        let pay = |work: usize, storage: usize| {
            steps.set(
                steps
                    .get()
                    .saturating_add(work)
                    .saturating_add(usize::from(storage != 0)),
            );
            bytes.set(bytes.get().saturating_add(storage));
            if steps.get() > crates::MAX_COMPONENTS {
                return Err("source export work budget exceeded");
            }
            if bytes.get().saturating_add(scratch) > crates::MAX_BYTES {
                return Err("source export retained and scratch budget exceeded");
            }
            Ok(())
        };
        let result = query(&pay);
        self.steps = steps.get();
        self.bytes = bytes.get();
        result
    }
}
