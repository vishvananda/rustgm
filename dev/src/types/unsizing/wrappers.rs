//! Sparse wrapper declaration data; actual trait and caller proofs live elsewhere.
use super::{ItemId, Rules, Store, TypeId};
use crate::{
    support::{
        hash::HashMap,
        retained::{self, Growth, Payment},
    },
    types::{Kind, arena::Identity, arguments::Argument, substitute::Bound},
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Field {
    pub source: TypeId,
    pub target: TypeId,
    pub phantom: bool,
}
#[derive(PartialEq, Eq)]
pub(crate) struct Schema {
    pub source: TypeId,
    pub target: TypeId,
    pub changed: u32,
    pub fields: Vec<Field>,
}
pub(super) struct Plans {
    pub(super) arena: Identity,
    pub(super) schemas: HashMap<ItemId, Schema>,
    pub(super) transfers: HashMap<(TypeId, TypeId), super::transfers::Plan>,
    hint_slots: HashMap<ItemId, Vec<bool>>,
    work: usize,
    bytes: usize,
}
impl Plans {
    fn retain_hint_slots(
        &mut self,
        types: &Store,
        schema: &Schema,
        pay: Payment<'_>,
    ) -> Result<(), &'static str> {
        let (Some(Kind::Adt(item, from)), Some(Kind::Adt(other, to))) =
            (types.kind(schema.source), types.kind(schema.target))
        else {
            return Err("nominal coercion hint lacks its endpoint applications");
        };
        let (from, to) = types
            .arguments(from)
            .zip(types.arguments(to))
            .filter(|(a, b)| item == other && a.len() == b.len())
            .ok_or("nominal coercion hint changed its endpoint inputs")?;
        let width = from.len();
        if !self.hint_slots.contains_key(&item) {
            let mut growth = Growth::map::<ItemId, Vec<bool>>(
                self.hint_slots.len(),
                self.hint_slots.capacity(),
                1,
            );
            growth.include(Growth::vector::<bool>(0, 0, width));
            self.charge(growth.work, growth.bytes)?;
            pay(growth.work, growth.bytes)?;
            self.hint_slots
                .try_reserve(1)
                .map_err(|_| "nominal coercion hint index allocation failed")?;
            let mut slots = Vec::new();
            slots
                .try_reserve_exact(width)
                .map_err(|_| "nominal coercion hint slot allocation failed")?;
            slots.resize(width, false);
            self.hint_slots.insert(item, slots);
        }
        self.charge(width, 0)?;
        pay(width, 0)?;
        let slots = self
            .hint_slots
            .get_mut(&item)
            .filter(|slots| slots.len() == width)
            .ok_or("nominal coercion hint changed its declaration arity")?;
        for (slot, (a, b)) in slots.iter_mut().zip(from.zip(to)) {
            *slot |= matches!((a, b), (Argument::Type(a), Argument::Type(b)) if a != b);
        }
        Ok(())
    }

    pub(super) fn charge(&mut self, work: usize, bytes: usize) -> Result<(), &'static str> {
        self.work = self.work.saturating_add(work);
        self.bytes = self.bytes.saturating_add(bytes);
        self.require_budget()
    }
    pub(super) fn require_budget(&self) -> Result<(), &'static str> {
        if self.work > 1_000_000 || self.bytes > 64 * 1024 * 1024 {
            return Err("nominal coercion recipe work/storage budget exceeded");
        }
        Ok(())
    }
}
impl Rules {
    pub(crate) fn retain_nominal_schema(
        &mut self,
        types: &mut Store,
        owner: ItemId,
        schema: Schema,
        pay: Payment<'_>,
    ) -> Result<(), &'static str> {
        if self.nominal.is_none() {
            pay(
                1,
                size_of::<Plans>().saturating_add(if types.existing_arena_identity().is_none() {
                    Identity::allocation_bytes()
                } else {
                    0
                }),
            )?;
            self.nominal = Some(Box::new(Plans {
                arena: types.arena_identity(),
                schemas: HashMap::default(),
                transfers: HashMap::default(),
                hint_slots: HashMap::default(),
                work: 0,
                bytes: size_of::<Plans>(),
            }));
        }
        let plans = self
            .nominal
            .as_deref_mut()
            .ok_or("nominal coercion recipe owner disappeared")?;
        if !types.has_identity(&plans.arena) {
            return Err("nominal coercion recipe belongs to another type store");
        }
        pay(1, 0)?;
        plans.charge(1, 0)?;
        if let Some(previous) = plans.schemas.get(&owner) {
            return if *previous == schema {
                Ok(())
            } else {
                Err("nominal coercion declaration recipe changed")
            };
        }
        if schema.changed as usize >= schema.fields.len()
            || types.kind(schema.source).is_none()
            || types.kind(schema.target).is_none()
        {
            return Err("nominal coercion declaration recipe is malformed");
        }
        let growth =
            Growth::map::<ItemId, Schema>(plans.schemas.len(), plans.schemas.capacity(), 1);
        plans.charge(
            growth.work,
            growth
                .bytes
                .saturating_add(retained::vector(&schema.fields)),
        )?;
        pay(growth.work, growth.bytes)?;
        plans
            .schemas
            .try_reserve(1)
            .map_err(|_| "nominal coercion declaration recipe allocation failed")?;
        plans.retain_hint_slots(types, &schema, pay)?;
        plans.schemas.insert(owner, schema);
        Ok(())
    }
    pub(crate) fn nominal_schema(
        &self,
        types: &Store,
        owner: ItemId,
    ) -> Result<Option<&Schema>, &'static str> {
        let Some(plans) = self.nominal.as_deref() else {
            return Ok(None);
        };
        if !types.has_identity(&plans.arena) {
            return Err("nominal coercion recipe belongs to another type store");
        }
        plans.require_budget()?;
        Ok(plans.schemas.get(&owner))
    }

    /// Declaration inputs that may differ across a checked wrapper coercion.
    /// The hint only controls which equations can run before argument checking.
    pub(crate) fn nominal_hint_slots(
        &self,
        types: &Store,
        owner: ItemId,
    ) -> Result<Option<&[bool]>, &'static str> {
        let Some(plans) = self.nominal.as_deref() else {
            return Ok(None);
        };
        if !types.has_identity(&plans.arena) {
            return Err("nominal coercion hint belongs to another type store");
        }
        plans.require_budget()?;
        Ok(plans.hint_slots.get(&owner).map(Vec::as_slice))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/transfer_budgets.rs"]
mod tests;

impl Schema {
    pub(crate) fn endpoints(
        &self,
        types: &mut Store,
        fold: &mut Bound,
        pay: Payment<'_>,
    ) -> Result<(TypeId, TypeId), &'static str> {
        let mut apply = |ty| match fold.apply_argument_charged(types, Argument::Type(ty), &pay)? {
            Argument::Type(ty) => Ok(ty),
            _ => Err("nominal coercion recipe lost its type argument"),
        };
        Ok((apply(self.source)?, apply(self.target)?))
    }
}
