//! Completed, debt-free dependency closures are independent of a requesting body.
use crate::{
    resolve::model::DefId,
    sema::{traits::implementations::Availability, values::Instance},
    types::parameters::predicates::Predicate,
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub(crate) struct Dependencies {
    pub(crate) work: crate::sema::model::DispatchWork,
    ready: HashSet<DefId>,
    declarations: HashSet<Instance>,
    validated: HashSet<Instance>,
    declaration_proofs: HashMap<Predicate, super::implementation::Target>,
    validated_proofs: HashMap<Predicate, super::implementation::Target>,
}
impl Dependencies {
    pub(super) fn selection(
        &self,
        availability: Availability,
        predicate: Predicate,
    ) -> Option<super::implementation::Target> {
        match availability {
            Availability::Declarations => self.declaration_proofs.get(&predicate),
            Availability::Validated => self.validated_proofs.get(&predicate),
        }
        .copied()
    }
    pub(super) fn certify(
        &mut self,
        availability: Availability,
        predicate: Predicate,
        target: super::implementation::Target,
    ) {
        match availability {
            Availability::Declarations => &mut self.declaration_proofs,
            Availability::Validated => &mut self.validated_proofs,
        }
        .insert(predicate, target);
    }
    pub(in crate::sema) fn complete(&mut self, definition: DefId) {
        self.ready.insert(definition);
    }
    pub(super) fn ready(&self, definition: DefId) -> bool {
        self.ready.contains(&definition)
    }
    pub(in crate::sema) fn contains(&self, availability: Availability, instance: Instance) -> bool {
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
