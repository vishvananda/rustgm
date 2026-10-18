//! Store-owned structural keys carry no lifetime proof or caller permission.
use super::{Store, parameters::predicates::Predicate, substitute::Bound};
#[cfg(test)]
#[path = "../../tests/invariants/shape_keys.rs"]
mod tests;

/// Lifetime-erased predicates are only comparable keys, never usable proof facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ShapeKey(Predicate);

impl Store {
    /// Share context-free canonical graph transformations across independent queries.
    /// The folder's cumulative work limit also bounds this derived index's storage.
    /// Query owners charge the returned incremental work to their own budget.
    pub(crate) fn shape_key(
        &mut self,
        predicate: Predicate,
    ) -> Result<(ShapeKey, usize), &'static str> {
        let mut shapes = self
            .shapes
            .take()
            .unwrap_or_else(|| Box::new(Bound::shape()));
        let before = shapes.steps;
        let result = shapes.predicate(self, predicate).map(ShapeKey);
        let work = shapes.steps.saturating_sub(before);
        self.shapes = Some(shapes);
        result.map(|key| (key, work))
    }

    #[cfg(test)]
    pub(crate) fn shape_key_steps(&self) -> usize {
        self.shapes.as_ref().map_or(0, |shapes| shapes.steps)
    }
}
