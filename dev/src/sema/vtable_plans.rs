//! Executable object plans are demanded from bodies whose source proofs have passed.
mod shapes;
mod tables;
mod upcasts;
use crate::{
    resolve::model::DefId,
    types::{TypeId, instance::Instance, substitute::Bound, traits::ApplicationId},
};
use std::{collections::HashMap, sync::Arc};
pub(crate) use tables::{complete_tables, prepare_table};
pub(crate) use upcasts::prepare_upcast;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct ShapeId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TableId(pub u32);

pub(crate) struct Table {
    pub concrete: TypeId,
    pub shape: ShapeId,
    pub methods: Vec<Option<Instance>>,
}

#[derive(Clone, Copy)]
pub(crate) struct Slot {
    pub method: DefId,
    pub parent: ApplicationId,
}
pub(crate) struct Shape {
    pub object: TypeId,
    pub slots: Vec<Slot>,
    indices: HashMap<(DefId, ApplicationId), u32>,
    pub upcasts: Vec<ShapeId>,
    upcast_indices: HashMap<ShapeId, u32>,
}

#[derive(Clone, Copy)]
pub(crate) enum Upcast {
    Identity,
    Table { source: ShapeId, index: u32 },
}

#[derive(Clone, Copy)]
pub(crate) struct Virtual {
    pub shape: ShapeId,
    pub slot: u32,
}

#[derive(Default)]
pub(crate) struct Objects {
    storage: Option<Box<Storage>>,
}
struct Storage {
    shapes: Vec<Arc<Shape>>,
    shape_ids: HashMap<TypeId, ShapeId>,
    calls: HashMap<Instance, Option<Virtual>>,
    tables: Vec<Table>,
    table_ids: HashMap<(TypeId, ShapeId), TableId>,
    conversions: HashMap<(TypeId, TypeId), TableId>,
    upcasts: HashMap<(TypeId, TypeId), Upcast>,
    shape_tables: Vec<Vec<TableId>>,
    pending: Vec<(TypeId, ShapeId)>,
    pending_keys: std::collections::HashSet<(TypeId, ShapeId)>,
    fresh: Vec<TableId>,
    erase: Bound,
    compatibility: super::dyn_compat::Queries,
    steps: usize,
}
impl Default for Storage {
    fn default() -> Self {
        Self {
            shapes: Vec::new(),
            shape_ids: HashMap::new(),
            calls: HashMap::new(),
            tables: Vec::new(),
            table_ids: HashMap::new(),
            conversions: HashMap::new(),
            upcasts: HashMap::new(),
            shape_tables: Vec::new(),
            pending: Vec::new(),
            pending_keys: std::collections::HashSet::new(),
            fresh: Vec::new(),
            erase: Bound::shape(),
            compatibility: super::dyn_compat::Queries::default(),
            steps: 0,
        }
    }
}
impl Objects {
    pub(crate) fn call(&self, instance: Instance) -> Option<Virtual> {
        self.storage
            .as_ref()?
            .calls
            .get(&instance)
            .copied()
            .flatten()
    }
    pub(crate) fn shape(&self, id: ShapeId) -> Option<&Shape> {
        self.storage
            .as_ref()?
            .shapes
            .get(id.0 as usize)
            .map(AsRef::as_ref)
    }
    pub(crate) fn table(&self, id: TableId) -> Option<&Table> {
        self.storage.as_ref()?.tables.get(id.0 as usize)
    }
    pub(crate) fn conversion(&self, concrete: TypeId, object: TypeId) -> Option<TableId> {
        self.storage
            .as_ref()?
            .conversions
            .get(&(concrete, object))
            .copied()
    }
    pub(crate) fn upcast(&self, source: TypeId, target: TypeId) -> Option<Upcast> {
        self.storage
            .as_ref()?
            .upcasts
            .get(&(source, target))
            .copied()
    }
    pub(crate) fn table_for_shape(&self, concrete: TypeId, shape: ShapeId) -> Option<TableId> {
        self.storage
            .as_ref()?
            .table_ids
            .get(&(concrete, shape))
            .copied()
    }
}

impl Storage {
    fn schedule(&mut self, concrete: TypeId, shape: ShapeId) -> Result<(), &'static str> {
        self.charge(1)?;
        let key = (concrete, shape);
        if !self.table_ids.contains_key(&key) && self.pending_keys.insert(key) {
            self.pending.push(key);
        }
        Ok(())
    }
    fn charge(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps > 1_000_000 {
            return Err("trait object plan work budget exceeded");
        }
        Ok(())
    }
}

/// The requesting instance walker has verified the source body's lifetime and
/// ownership contracts. Erasure here chooses runtime layout, never permissions.
pub(crate) fn prepare_call(
    report: &crate::resolve::expansion::Report,
    facts: &mut super::model::Declarations,
    instance: Instance,
) -> Result<bool, String> {
    if let Some(value) = facts
        .objects
        .storage
        .as_ref()
        .and_then(|s| s.calls.get(&instance))
    {
        return Ok(value.is_some());
    }
    let Some((method, predicate)) = super::dispatch::object_method(facts, instance.ty())? else {
        return Ok(false);
    };
    let storage = facts.objects.storage.get_or_insert_with(Default::default);
    let before = storage.erase.steps;
    let predicate = storage.erase.predicate(&mut facts.types, predicate)?;
    storage.charge(storage.erase.steps.saturating_sub(before).saturating_add(1))?;
    let crate::types::parameters::predicates::Bound::Trait(parent) = predicate.bound else {
        return Err("object callable requires a complete parent trait".into());
    };
    let shape = shapes::prepare(report, facts, predicate.ty)?;
    let slot = facts
        .objects
        .shape(shape)
        .ok_or("missing object method shape")?
        .indices
        .get(&(method, parent))
        .copied();
    let value = slot.map(|slot| Virtual { shape, slot });
    facts
        .objects
        .storage
        .as_mut()
        .ok_or("missing object method plans")?
        .calls
        .insert(instance, value);
    Ok(value.is_some())
}
