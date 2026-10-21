//! A mapped environment charges copied payload, excluding unchanged declaration tables.
use super::Environment;
use crate::{support::retained::Allocation, types::parameters::premise_index};
use std::sync::Arc;
#[derive(Default)]
pub(crate) struct Retained {
    base: bool,
    index: premise_index::Retained,
}
impl Environment {
    pub(crate) fn retain_owned_from(&self, shared: &Self, observer: &mut Retained) -> Allocation {
        let mut total = Allocation::default();
        if !observer.base {
            observer.base = true;
            total.boxed(self);
            if !Arc::ptr_eq(&self.parameters, &shared.parameters) {
                total.boxed(self.parameters.as_ref());
                total.map(&self.parameters);
            }
            if let Some(value) = fresh(self.assumptions.as_ref(), shared.assumptions.as_ref()) {
                total.boxed(value);
                total.map(&value.predicates);
                total.set(&value.sized);
            }
            if let Some(value) = fresh(self.builtins.as_ref(), shared.builtins.as_ref()) {
                total.boxed(value);
                total.include(value.allocation());
            }
            if let Some(value) = fresh(self.regions.as_ref(), shared.regions.as_ref()) {
                total.boxed(value);
                total.include(value.allocation());
            }
            if let Some(value) = fresh(self.traits.as_ref(), shared.traits.as_ref()) {
                total.boxed(value);
                total.include(value.allocation());
            }
            if let Some(value) = fresh(self.equalities.as_ref(), shared.equalities.as_ref()) {
                total.boxed(value);
                total.include(value.allocation());
            }
            if let Some(value) = fresh(
                self.projection_promises.as_ref(),
                shared.projection_promises.as_ref(),
            ) {
                total.boxed(value);
                total.map(value);
            }
        }
        total.visits = total.visits.saturating_add(1);
        if let Some(value) = fresh(self.traits.as_ref(), shared.traits.as_ref()) {
            total.include(value.retain_index(&mut observer.index));
        }
        total
    }
}
fn fresh<'a, T>(value: Option<&'a Arc<T>>, shared: Option<&Arc<T>>) -> Option<&'a T> {
    value
        .filter(|value| shared.is_none_or(|shared| !Arc::ptr_eq(value, shared)))
        .map(Arc::as_ref)
}
