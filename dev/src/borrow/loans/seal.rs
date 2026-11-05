//! A transient cleanup seal retains current arenas and actual Copy proof ownership.
#[cfg(test)]
#[path = "../../../tests/invariants/clseal.rs"]
mod tests;
use crate::{
    body::{ir::Body, regions::BuiltinRequirements},
    borrow::{Environment, regions},
    types::arena::Identity,
};
use std::{ptr::NonNull, sync::Arc};
pub(crate) struct Seal {
    body: NonNull<Body>,
    builtins: Option<Arc<BuiltinRequirements>>,
    types: Option<Identity>,
    definitions: Option<Identity>,
    owner: Option<Arc<()>>,
}
impl Seal {
    pub(crate) fn new<S: std::hash::BuildHasher>(
        environment: &Environment<'_, S>,
        body: &Body,
        contract: &regions::Body,
        owner: Option<Arc<()>>,
    ) -> Self {
        Self {
            body: NonNull::from(body),
            builtins: contract.builtins.clone(),
            types: environment.types.existing_arena_identity().cloned(),
            definitions: environment.definitions.existing_arena_identity().cloned(),
            owner,
        }
    }
    pub(crate) fn validate<S: std::hash::BuildHasher>(
        &self,
        environment: &Environment<'_, S>,
        body: &Body,
        contract: &regions::Body,
        owner: Option<&Arc<()>>,
    ) -> Result<(), &'static str> {
        if self.body != NonNull::from(body)
            || self
                .types
                .as_ref()
                .is_some_and(|id| !environment.types.has_identity(id))
            || self
                .definitions
                .as_ref()
                .is_some_and(|id| !environment.definitions.has_identity(id))
            || !same_arc(self.owner.as_ref(), owner)
            || contract.key != body.key()
            || contract.source != body.source
            || contract.instance != body.instance
        {
            return Err("initialized cleanup plan changed its arena, verifier or body");
        }
        let before = self.builtins.as_deref();
        let after = contract.builtins.as_deref();
        if !same_arc(
            before.and_then(|r| r.environment.as_ref()),
            after.and_then(|r| r.environment.as_ref()),
        ) || !same_arc(
            before.and_then(|r| r.source_copy.as_ref()),
            after.and_then(|r| r.source_copy.as_ref()),
        ) || before.map(|r| &r.predicates) != after.map(|r| &r.predicates)
        {
            return Err("initialized cleanup plan changed its actual Copy evidence or environment");
        }
        Ok(())
    }
    pub(crate) fn require_arenas<S: std::hash::BuildHasher>(
        environment: &Environment<'_, S>,
    ) -> Result<(), &'static str> {
        if environment.types.existing_arena_identity().is_none()
            || environment.definitions.existing_arena_identity().is_none()
        {
            return Err("staged cleanup requires its actual current arenas");
        }
        Ok(())
    }
}
fn same_arc<T>(left: Option<&Arc<T>>, right: Option<&Arc<T>>) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        _ => false,
    }
}
