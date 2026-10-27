//! Store-owned structural keys carry no lifetime proof or caller permission.
use super::{Store, parameters::predicates::Predicate, substitute::Bound};
#[cfg(test)]
#[path = "../../tests/invariants/shape_keys.rs"]
mod tests;

/// Lifetime-erased predicates are only comparable keys, never usable proof facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ShapeKey(Predicate);

impl Store {
    /// Canonical storage is distinct from the fold's own memo/scratch. Bound
    /// calls this immediately before the same existing shape rebuild.
    pub(in crate::types) fn shape_rebuild_growth(
        &self,
        kind: super::Kind,
    ) -> Result<crate::support::retained::Growth, &'static str> {
        use super::{Kind, Properties, TypeId};
        use crate::support::retained::Growth;
        let mut growth = Growth::vector::<Kind>(self.kinds.len(), self.kinds.capacity(), 1);
        growth.include(Growth::vector::<Properties>(
            self.properties.len(),
            self.properties.capacity(),
            1,
        ));
        growth.include(Growth::map::<Kind, TypeId>(
            self.index.len(),
            self.index.capacity(),
            1,
        ));
        match kind {
            Kind::Tuple(list)
            | Kind::Function {
                parameters: list, ..
            } => {
                let count = self
                    .list(list)
                    .ok_or("invalid shape rebuild child list")?
                    .len();
                growth.include(self.children.interning_growth(count));
            }
            Kind::Adt(_, arguments)
            | Kind::Opaque(_, arguments)
            | Kind::FunctionItem { arguments, .. }
            | Kind::Closure { arguments, .. }
            | Kind::Dynamic { arguments, .. } => {
                growth.include(self.shape_argument_growth(&[arguments])?);
            }
            Kind::Projection(application) => {
                let trait_ = self
                    .trait_application(application.trait_application)
                    .ok_or("invalid shape rebuild trait application")?;
                growth.include(
                    self.shape_argument_growth(&[trait_.arguments, application.arguments])?,
                );
                growth.include(self.trait_application_growth());
            }
            _ => {}
        }
        Ok(growth)
    }

    fn shape_argument_growth(
        &self,
        arguments: &[super::GenericArgs],
    ) -> Result<crate::support::retained::Growth, &'static str> {
        let (mut regions, mut mixed) = ((0_usize, 0_usize), (0_usize, 0_usize));
        for &arguments in arguments {
            let arguments = self
                .arguments(arguments)
                .ok_or("invalid shape rebuild arguments")?;
            let count = arguments.len();
            let totals = match arguments {
                super::arguments::Arguments::Lifetimes(_) => &mut regions,
                super::arguments::Arguments::Mixed(_) => &mut mixed,
            };
            totals.0 = totals.0.saturating_add(count);
            totals.1 = totals.1.saturating_add(usize::from(count != 0));
        }
        let mut growth = self.regions.interning_batch_growth(regions.0, regions.1);
        growth.include(self.mixed_argument_batch_growth(mixed.0, mixed.1));
        growth.work = growth
            .work
            .saturating_add(regions.0)
            .saturating_add(mixed.0)
            .saturating_add(arguments.len());
        Ok(growth)
    }

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
