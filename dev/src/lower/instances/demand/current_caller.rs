//! Dependency requests retain their actual sealed caller, separately from diagnostics.
use super::{Checked, Declarations, proof};
use crate::{
    body::ir::{Body, Owner, ProofSite},
    resolve::model::DefId,
    types::parameters::environment::Environment,
};
use std::sync::Arc;
#[derive(Clone, Copy)]
pub(in crate::lower::instances) struct Caller(u32);
const GLOBAL: u32 = 1 << 31;
impl Caller {
    pub(in crate::lower::instances) const ROOT: Self = Self(0);
    pub(in crate::lower::instances) fn body(index: usize) -> Result<Self, &'static str> {
        let index = u32::try_from(index).map_err(|_| "sealed caller index exceeds 32 bits")?;
        index
            .checked_add(1)
            .filter(|id| *id < GLOBAL)
            .map(Self)
            .ok_or("sealed caller index exceeds compact request space")
    }
    pub(in crate::lower::instances) fn global(definition: DefId) -> Result<Self, &'static str> {
        definition
            .0
            .checked_add(1)
            .filter(|id| *id < GLOBAL)
            .map(|id| Self(id | GLOBAL))
            .ok_or("initializer caller exceeds compact request space")
    }
    pub(in crate::lower::instances) fn environment(
        self,
        facts: &mut Declarations,
        templates: &Checked<'_>,
        program: &proof::Builder,
        source: ProofSite,
    ) -> Result<Option<Arc<Environment>>, &'static str> {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(16);
        if self.0 == 0 {
            return Ok(None);
        }
        if self.0 & GLOBAL != 0 {
            let owner = DefId((self.0 & !GLOBAL) - 1);
            let initializer = facts
                .initializer_index(owner)
                .and_then(|index| facts.initializers.get(index))
                .ok_or("global caller has no actual initializer")?;
            if source.source() != Some(initializer.value) || !initializer.type_checked() {
                return Err("global caller changed its checked source or owner");
            }
            return facts
                .generics
                .sealed_caller_environment(owner)
                .map(Option::<&Arc<Environment>>::cloned);
        }
        let body = program.body((self.0 - 1) as usize)?;
        if body.source.root_site() != source {
            return Err("sealed caller changed its actual root provenance");
        }
        if let Some(environment) = templates.environment_for_body(body)? {
            return Ok(Some(Arc::clone(environment)));
        }
        let owner = ordinary_owner(facts, body)?;
        facts
            .generics
            .sealed_caller_environment(owner)
            .map(Option::<&Arc<Environment>>::cloned)
    }
}
fn ordinary_owner(facts: &Declarations, body: &Body) -> Result<DefId, &'static str> {
    let source = body
        .source
        .source()
        .ok_or("imported caller requires its actual checked environment")?;
    match body.owner {
        Owner::Function { definition, .. } => {
            let function = facts
                .function_indices
                .get(&definition)
                .and_then(|&index| facts.functions.get(index))
                .ok_or("ordinary caller has no actual function declaration")?;
            if function.body != Some(source) || !facts.typed_function_bodies.contains(&definition) {
                return Err("ordinary caller changed its completed source body");
            }
            Ok(definition)
        }
        Owner::Closure { identity, .. } => {
            let closure = facts
                .closures
                .identity(identity)
                .ok_or("ordinary caller has no actual closure")?;
            if closure.semantic_source() != source {
                return Err("ordinary closure caller changed source");
            }
            closure
                .owner
                .ok_or("ordinary closure caller requires its actual declaration scope")
        }
        Owner::Initializer { definition, .. }
        | Owner::InitializerInstance {
            instance: crate::sema::values::Instance { definition, .. },
            ..
        } => {
            let initializer = facts
                .initializer_index(definition)
                .and_then(|index| facts.initializers.get(index))
                .ok_or("ordinary caller has no actual initializer")?;
            if initializer.semantic_source() != source || !initializer.type_checked() {
                return Err("ordinary initializer caller changed completed source");
            }
            Ok(definition)
        }
        Owner::AnonymousConstant | Owner::AnonymousInstance(_) => facts
            .inline_constants
            .as_deref()
            .and_then(|registry| registry.get(source))
            .and_then(|body| body.declaration)
            .ok_or("anonymous caller requires its actual declaration scope"),
        Owner::ClosureAdapter { .. } => {
            Err("adapter caller requires its checked source environment")
        }
    }
}
