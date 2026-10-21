//! Source conversions supply captured entry evidence before physical table reuse.
use super::{Method, Target};
use crate::{
    sema::{model::Declarations, traits::object_bounds},
    support::call_traits::CallTrait,
    types::{TypeId, closure::Adapter, instance::Instance, parameters::predicates::Bound},
};

/// A physical target description, not a source-proof capability. Lowering must
/// find the exact sealed source/adapter body and validate its execution ABI.
#[derive(Clone, Copy)]
pub(crate) struct Captured {
    pub instance: Instance,
    pub requested: CallTrait,
    pub execution: TypeId,
    pub adapter: Option<Adapter>,
}

pub(crate) fn captured_permissions(
    facts: &mut Declarations,
    object: TypeId,
) -> Result<[bool; 2], &'static str> {
    let (bounds, steps) = object_bounds::get(facts, object)?;
    let mut permissions = [false; 2];
    for &clause in &bounds.applications {
        let predicate = object_bounds::graph::quantified_value(&facts.types, clause)?;
        let Bound::Trait(parent) = predicate.bound else {
            return Err("captured object member requires a trait application");
        };
        let item = facts
            .types
            .trait_application(parent)
            .ok_or("missing captured object application")?
            .item;
        match facts
            .associated
            .callable(crate::resolve::model::DefId(item.0))
        {
            Some(CallTrait::Fn) => permissions[0] = true,
            Some(CallTrait::FnMut) => permissions[1] = true,
            _ => {}
        }
    }
    facts.body_work.canonical_steps += steps + bounds.applications.len();
    Ok(permissions)
}

pub(crate) fn prepare_captured(
    facts: &mut Declarations,
    concrete: TypeId,
    entries: [Option<Captured>; 2],
) -> Result<(), &'static str> {
    let storage = facts.objects.storage.get_or_insert_with(Default::default);
    let before = storage.erase.steps;
    let erased = storage.erase.apply(&mut facts.types, concrete)?;
    storage.charge(storage.erase.steps.saturating_sub(before).saturating_add(1))?;
    for entry in entries.into_iter().flatten() {
        if entry.instance.ty() != concrete || entry.requested == CallTrait::FnOnce {
            return Err("captured table entry differs from its checked source conversion");
        }
        storage
            .captured_targets
            .entry((erased, entry.requested))
            .or_insert(entry);
    }
    Ok(())
}

pub(super) fn target(
    facts: &Declarations,
    concrete: TypeId,
    method: Method,
) -> Result<Option<Target>, &'static str> {
    let Method::Callable(kind) = method else {
        return Err("captured callable target has a declared method slot");
    };
    if kind == CallTrait::FnOnce {
        return Ok(None);
    }
    facts
        .objects
        .storage
        .as_ref()
        .and_then(|storage| storage.captured_targets.get(&(concrete, kind)))
        .copied()
        .map(Target::Captured)
        .map(Some)
        .ok_or("captured object target lacks checked conversion entry evidence")
}
