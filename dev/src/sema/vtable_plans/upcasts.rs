//! Upcast plans link complete erased shapes only after source evidence is sealed.
use super::{Upcast, shapes};
use crate::{resolve::expansion::Report, sema::model::Declarations, types::TypeId};
use std::sync::Arc;

pub(crate) fn prepare_upcast(
    report: &Report,
    facts: &mut Declarations,
    source: TypeId,
    target: TypeId,
) -> Result<(), String> {
    if facts.objects.upcast(source, target).is_some() {
        return Ok(());
    }
    let key = (source, target);
    let storage = facts.objects.storage.get_or_insert_with(Default::default);
    let before = storage.erase.steps;
    let source = storage.erase.apply(&mut facts.types, source)?;
    let target = storage.erase.apply(&mut facts.types, target)?;
    storage.charge(storage.erase.steps.saturating_sub(before).saturating_add(1))?;
    let source = shapes::prepare(report, facts, source)?;
    let target = shapes::prepare(report, facts, target)?;
    let storage = facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing upcast storage")?;
    let plan = if source == target {
        Upcast::Identity
    } else {
        let shape = storage
            .shapes
            .get_mut(source.0 as usize)
            .and_then(Arc::get_mut)
            .ok_or("object shape is borrowed during upcast discovery")?;
        let index = if let Some(&index) = shape.upcast_indices.get(&target) {
            index
        } else {
            let index =
                u32::try_from(shape.upcasts.len()).map_err(|_| "too many upcast targets")?;
            shape.upcasts.push(target);
            shape.upcast_indices.insert(target, index);
            let concrete = storage
                .shape_tables
                .get(source.0 as usize)
                .ok_or("missing upcast source tables")?
                .iter()
                .map(|id| storage.tables[id.0 as usize].concrete)
                .collect::<Vec<_>>();
            for ty in concrete {
                storage.schedule(ty, target)?;
            }
            index
        };
        Upcast::Table { source, index }
    };
    storage.upcasts.insert(key, plan);
    Ok(())
}
