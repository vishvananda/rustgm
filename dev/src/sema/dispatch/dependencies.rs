//! Completed, debt-free dependency closures are independent of a requesting body.
use crate::{
    resolve::model::DefId,
    sema::{traits::implementations::Availability, values::Instance},
};
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct Dependencies {
    ready: HashSet<DefId>,
    declarations: HashSet<Instance>,
    validated: HashSet<Instance>,
}
impl Dependencies {
    pub(in crate::sema) fn complete(&mut self, definition: DefId) {
        self.ready.insert(definition);
    }
    pub(super) fn ready(&self, definition: DefId) -> bool {
        self.ready.contains(&definition)
    }
    pub(super) fn contains(&self, availability: Availability, instance: Instance) -> bool {
        match availability {
            Availability::Declarations => self.declarations.contains(&instance),
            Availability::Validated => self.validated.contains(&instance),
        }
    }
    pub(super) fn insert(&mut self, availability: Availability, instance: Instance) {
        match availability {
            Availability::Declarations => self.declarations.insert(instance),
            Availability::Validated => self.validated.insert(instance),
        };
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/dispatch_dependencies.rs"]
mod tests;
