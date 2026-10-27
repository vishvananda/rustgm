//! Store-owned structural keys carry no lifetime proof or caller permission.
use super::{Store, parameters::predicates::Predicate, substitute::Bound};
#[cfg(test)]
#[path = "../../tests/invariants/shape_keys.rs"]
mod tests;

/// Lifetime-erased predicates are only comparable keys, never usable proof facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ShapeKey(Predicate);

impl Store {
    /// Charge the same persistent shape producer before its owned allocations.
    pub(crate) fn shape_key_charged(
        &mut self,
        predicate: Predicate,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(ShapeKey, usize, usize), &'static str> {
        // The actual shape action has no nested fold owner. Its allocation
        // observer uses one four-slot traversal stack, never a type-DAG walk.
        charge(1, 4 * size_of::<&Bound>())?;
        let before_bytes = self.shape_registration_bytes();
        let mut shapes = if let Some(shapes) = self.shapes.take() {
            shapes
        } else {
            charge(1, size_of::<Bound>())?;
            Box::new(Bound::shape())
        };
        let before = shapes.steps;
        let result = shapes
            .predicate_charged(self, predicate, charge)
            .map(ShapeKey);
        let work = shapes.steps.saturating_sub(before);
        self.shapes = Some(shapes);
        let retained = self.shape_registration_bytes().saturating_sub(before_bytes);
        result.map(|key| (key, work, retained))
    }

    fn shape_registration_bytes(&self) -> usize {
        self.node_table_allocation()
            .bytes
            .saturating_add(self.children.allocation().bytes)
            .saturating_add(self.regions.allocation().bytes)
            .saturating_add(self.trait_input_bytes())
            .saturating_add(self.shapes.as_ref().map_or(0, |shapes| {
                size_of::<Bound>().saturating_add(shapes.retained_allocation().0)
            }))
    }

    /// Share context-free canonical graph transformations across independent queries.
    /// The folder's cumulative work limit also bounds this derived index's storage.
    /// Query owners charge the returned incremental work to their own budget.
    pub(crate) fn shape_key(
        &mut self,
        predicate: Predicate,
    ) -> Result<(ShapeKey, usize), &'static str> {
        self.predicate_shape(predicate, false)
    }

    /// Fresh lifetime expansion is an inductive cutoff, not positive evidence.
    /// Preserve static slots so a constant-lifetime cycle can close independently
    /// of the generic lifetime application that first requested it.
    pub(crate) fn auto_shape_key(
        &mut self,
        predicate: Predicate,
    ) -> Result<(ShapeKey, usize), &'static str> {
        self.predicate_shape(predicate, true)
    }

    fn predicate_shape(
        &mut self,
        predicate: Predicate,
        auto: bool,
    ) -> Result<(ShapeKey, usize), &'static str> {
        let mut shapes = if auto {
            self.auto_shapes
                .take()
                .unwrap_or_else(|| Box::new(Bound::auto_shape()))
        } else {
            self.shapes
                .take()
                .unwrap_or_else(|| Box::new(Bound::shape()))
        };
        let before = shapes.steps;
        let result = shapes.predicate(self, predicate).map(ShapeKey);
        let work = shapes.steps.saturating_sub(before);
        if auto {
            self.auto_shapes = Some(shapes);
        } else {
            self.shapes = Some(shapes);
        }
        result.map(|key| (key, work))
    }

    #[cfg(test)]
    pub(crate) fn shape_key_steps(&self) -> usize {
        self.shapes.as_ref().map_or(0, |shapes| shapes.steps)
    }
}
