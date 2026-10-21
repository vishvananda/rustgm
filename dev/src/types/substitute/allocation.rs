//! A fold's owned caches retain IDs, while immutable store graphs stay shared.
use super::{Action, Bound};
use crate::support::retained::Allocation;
impl Bound {
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
