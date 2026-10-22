//! Runtime slots retain complete trait inputs while sharing repeated supertraits.
use super::{Method, Shape, ShapeId, Slot};
use crate::support::hash::HashMap;
use crate::{
    resolve::{expansion::Report, model::DefId},
    sema::{model::Declarations, traits::object_bounds},
    types::{Kind, TypeId, parameters::predicates::Bound},
};
use std::sync::Arc;

pub(super) fn prepare(
    report: &Report,
    facts: &mut Declarations,
    object: TypeId,
) -> Result<ShapeId, String> {
    if let Some(&id) = facts
        .objects
        .storage
        .as_ref()
        .and_then(|s| s.shape_ids.get(&object))
    {
        return Ok(id);
    }
    if !matches!(facts.types.kind(object), Some(Kind::Dynamic { .. })) {
        return Err("vtable shape requires an object type".into());
    }
    let (bounds, steps) = object_bounds::get(facts, object)?;
    let mut queries = std::mem::take(
        &mut facts
            .objects
            .storage
            .get_or_insert_with(Default::default)
            .compatibility,
    );
    let result = collect(report, facts, object, &bounds.applications, &mut queries);
    let storage = facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing object shape storage")?;
    storage.compatibility = queries;
    let shape = result?;
    storage.charge(steps.saturating_add(shape.slots.len()))?;
    let id = ShapeId(u32::try_from(storage.shapes.len()).map_err(|_| "too many object shapes")?);
    storage.shapes.push(Arc::new(shape));
    storage.shape_tables.push(Vec::new());
    storage.shape_ids.insert(object, id);
    Ok(id)
}

fn collect(
    report: &Report,
    facts: &mut Declarations,
    object: TypeId,
    applications: &[crate::types::parameters::clauses::Clause],
    queries: &mut crate::sema::dyn_compat::Queries,
) -> Result<Shape, String> {
    let mut shape = Shape {
        object,
        slots: Vec::new(),
        indices: HashMap::default(),
        upcasts: Vec::new(),
        upcast_indices: HashMap::default(),
    };
    for &clause in applications {
        let predicate = object_bounds::graph::quantified_value(&facts.types, clause)?;
        let Bound::Trait(parent) = predicate.bound else {
            return Err("object slot owner requires a trait".into());
        };
        let storage = facts
            .objects
            .storage
            .as_mut()
            .ok_or("missing object erasure")?;
        let before = storage.erase.steps;
        let parent = storage.erase.trait_application(&mut facts.types, parent)?;
        storage.charge(storage.erase.steps.saturating_sub(before).saturating_add(1))?;
        let owner = facts
            .types
            .trait_application(parent)
            .ok_or("missing object slot owner")?
            .item;
        if let Some(kind) = facts.associated.callable(DefId(owner.0)) {
            insert(&mut shape, Method::Callable(kind), parent)?;
            continue;
        }
        let contract = queries
            .demand(report, facts, DefId(owner.0))
            .map_err(|e| e.to_string())?;
        for &method in &contract.methods {
            insert(&mut shape, Method::Declared(method), parent)?;
        }
    }
    Ok(shape)
}

fn insert(
    shape: &mut Shape,
    method: Method,
    parent: crate::types::traits::ApplicationId,
) -> Result<(), &'static str> {
    if let std::collections::hash_map::Entry::Vacant(entry) = shape.indices.entry((method, parent))
    {
        let slot = u32::try_from(shape.slots.len()).map_err(|_| "too many virtual methods")?;
        entry.insert(slot);
        shape.slots.push(Slot { method, parent });
    }
    Ok(())
}
