//! A fold's owned caches retain IDs, while immutable store graphs stay shared.
use super::{Action, Bound};
use crate::support::retained::Allocation;
impl Bound {
    /// Conservative scratch/cache reservation for validated scalar/parameter roots.
    /// The caller validates the complete bindings and leaf domain before allocation.
    pub(crate) fn parameter_leaf_allocation_bound(bindings: usize, roots: usize) -> usize {
        use crate::{
            support::retained::new_map,
            types::{TypeId, arguments::Argument, parameters::ParameterId},
        };
        size_of::<Self>()
            .saturating_add(size_of::<super::parameters::Bindings>())
            .saturating_add(new_map::<ParameterId, Argument>(bindings))
            .saturating_add(new_map::<super::Key, TypeId>(roots))
            .saturating_add(
                4 * roots.saturating_mul(size_of::<(super::Key, bool)>() + size_of::<TypeId>()),
            )
    }
    pub(crate) fn retained_allocation(&self) -> (usize, usize) {
        let mut total = Allocation::default();
        let mut work = vec![self];
        while let Some(value) = work.pop() {
            total.vector(&value.arguments);
            total.map(&value.completed);
            total.vector(&value.work);
            total.vector(&value.children);
            total.vector(&value.regions);
            total.vector(&value.generic);
            if let Some(parameters) = &value.parameters {
                total.boxed(parameters.as_ref());
                total.include(parameters.allocation());
            }
            if let Some(applications) = &value.applications {
                total.boxed(applications.as_ref());
                total.include(applications.allocation());
            }
            match &value.action {
                Action::Items(value) => {
                    total.boxed(value.as_ref());
                    value.allocation(&mut total, &mut work);
                }
                Action::Universals(value) => {
                    total.boxed(value.as_ref());
                    total.include(value.allocation());
                }
                Action::Regions(value) => {
                    total.boxed(value.as_ref());
                    total.include(value.allocation());
                }
                Action::BodyScope(aliases) => {
                    total.boxed(aliases.as_ref());
                    total.include(aliases.allocation());
                }
                Action::Remove
                | Action::Shift(_)
                | Action::Lower(_)
                | Action::Shape { .. }
                | Action::Normalize => {}
            }
        }
        (total.bytes, total.visits)
    }
}
