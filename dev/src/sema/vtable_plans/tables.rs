//! Share concrete tables by complete erased object shape, preserving typed targets.
use super::{ShapeId, Table, TableId, shapes};
use crate::{resolve::expansion::Report, sema::model::Declarations, types::TypeId};

pub(crate) fn prepare_table(
    report: &Report,
    facts: &mut Declarations,
    concrete: TypeId,
    object: TypeId,
) -> Result<(), String> {
    if facts.objects.conversion(concrete, object).is_some() {
        return Ok(());
    }
    let key = (concrete, object);
    let storage = facts.objects.storage.get_or_insert_with(Default::default);
    let before = storage.erase.steps;
    let concrete = storage.erase.apply(&mut facts.types, concrete)?;
    let object = storage.erase.apply(&mut facts.types, object)?;
    storage.charge(storage.erase.steps.saturating_sub(before).saturating_add(1))?;
    let shape = shapes::prepare(report, facts, object)?;
    let id = intern(facts, concrete, shape)?;
    facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing conversion plans")?
        .conversions
        .insert(key, id);
    Ok(())
}

/// New upcast edges and new concrete types meet in one deduplicated worklist.
/// No existing source body or previously selected method table is rescanned.
pub(crate) fn complete_tables(facts: &mut Declarations) -> Result<Vec<TableId>, String> {
    while let Some((concrete, shape)) = facts.objects.storage.as_mut().and_then(|s| s.pending.pop())
    {
        facts
            .objects
            .storage
            .as_mut()
            .ok_or("missing pending vtables")?
            .pending_keys
            .remove(&(concrete, shape));
        intern(facts, concrete, shape)?;
    }
    Ok(facts
        .objects
        .storage
        .as_mut()
        .map_or_else(Vec::new, |s| std::mem::take(&mut s.fresh)))
}

fn intern(facts: &mut Declarations, concrete: TypeId, shape: ShapeId) -> Result<TableId, String> {
    let storage = facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing object table storage")?;
    if let Some(&id) = storage.table_ids.get(&(concrete, shape)) {
        return Ok(id);
    }
    let slots = storage
        .shapes
        .get(shape.0 as usize)
        .ok_or("missing table shape")?
        .clone();
    facts.aggregates.prepare(&mut facts.types, concrete)?;
    let mut methods = Vec::with_capacity(slots.slots.len());
    for &slot in &slots.slots {
        methods.push(crate::sema::dispatch::table_method(facts, concrete, slot)?);
    }
    let storage = facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing object table storage")?;
    storage.charge(methods.len().saturating_add(1))?;
    let id = TableId(u32::try_from(storage.tables.len()).map_err(|_| "too many vtables")?);
    storage.tables.push(Table {
        concrete,
        shape,
        methods,
    });
    storage.table_ids.insert((concrete, shape), id);
    storage
        .shape_tables
        .get_mut(shape.0 as usize)
        .ok_or("missing shape table index")?
        .push(id);
    storage.fresh.push(id);
    for &target in &slots.upcasts {
        storage.schedule(concrete, target)?;
    }
    Ok(id)
}
