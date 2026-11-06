//! Sparse wrapper declaration data; actual trait and caller proofs live elsewhere.
use super::{ItemId, Rules, Store, TypeId};
use crate::{
    support::{
        hash::HashMap,
        retained::{self, Growth, Payment},
    },
    types::{arena::Identity, arguments::Argument, substitute::Bound},
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
    work: usize,
    bytes: usize,
}
impl Plans {
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
