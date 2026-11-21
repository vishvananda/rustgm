//! Closed public-layout recipes are data; selection and lifetime debts stay in bodies.
use super::{Definitions, ItemId, Kind, Rules, Store, TypeId, aggregate};
use crate::support::retained::{self, Growth, Payment};

/// The pointee of a nominal conversion's changed pointer field: a reference
/// or raw pointer, bare or under a pattern type such as core's
/// `*const T is !null`, which keeps its base's representation.
pub(crate) fn terminal_pointee(types: &Store, ty: TypeId) -> Option<TypeId> {
    let ty = match types.kind(ty)? {
        Kind::Pattern { base, .. } => base,
        _ => ty,
    };
    match types.kind(ty)? {
        Kind::Reference { referent, .. } => Some(referent),
        Kind::Pointer { pointee, .. } => Some(pointee),
        _ => None,
    }
}

/// Find the array that supplies slice metadata for an already proved unsizing.
/// Each enclosing struct keeps its owner and contributes its prepared last field.
pub(crate) fn slice_terminal(
    types: &Store,
    aggregates: &Definitions,
    (mut source, mut target): (TypeId, TypeId),
    mut charge: impl FnMut() -> Result<(), &'static str>,
) -> Result<Option<TypeId>, &'static str> {
    let tail = |ty| {
        let shape = aggregates.shape(types, ty)?;
        shape
            .variant(0)
            .and_then(|variant| types.list(variant.fields))
            .and_then(|fields| fields.last())
            .copied()
            .ok_or("nominal slice conversion lost its prepared struct tail")
    };
    loop {
        charge()?;
        match (types.kind(source), types.kind(target)) {
            (Some(Kind::Array { .. }), Some(Kind::Slice(_))) => return Ok(Some(source)),
            (Some(Kind::Adt(a, _)), Some(Kind::Adt(b, _)))
                if a == b
                    && aggregates
                        .get(a)
                        .is_some_and(|definition| definition.kind == aggregate::Kind::Struct) =>
            {
                (source, target) = (tail(source)?, tail(target)?);
            }
            _ => return Ok(None),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    /// The terminal array supplies the length of the target slice, including a struct tail.
    Slice(TypeId),
    /// A thin pointer to `concrete` becomes a pointer to the trait object
    /// `object`, whose metadata is the concrete type's vtable.
    Object {
        concrete: TypeId,
        object: TypeId,
    },
    /// A wide pointer keeps its data address and selects the target object table.
    Upcast {
        source: TypeId,
        target: TypeId,
    },
    Nested {
        source: TypeId,
        target: TypeId,
    },
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Span {
    pub source: TypeId,
    pub target: TypeId,
    pub from: u64,
    pub to: u64,
    pub bytes: u64,
    pub align: u32,
}
#[derive(PartialEq, Eq)]
pub(crate) struct Plan {
    pub owner: ItemId,
    pub source: TypeId,
    pub target: TypeId,
    pub changed: u32,
    pub fields: Vec<Span>,
    pub step: Step,
}
#[cfg(test)]
const _: () = {
    assert!(size_of::<Step>() == 12);
    assert!(size_of::<Span>() == 40);
    assert!(size_of::<Plan>() == 56);
};
impl Rules {
    pub(crate) fn nominal_plan(
        &self,
        types: &Store,
        source: TypeId,
        target: TypeId,
    ) -> Result<Option<&Plan>, &'static str> {
        let Some(plans) = self.nominal.as_deref() else {
            return Ok(None);
        };
        if !types.has_identity(&plans.arena) {
            return Err("nominal conversion plan belongs to another type store");
        }
        plans.require_budget()?;
        let plan = plans.transfers.get(&(source, target));
        if let Some(plan) = plan {
            let schema = plans
                .schemas
                .get(&plan.owner)
                .ok_or("nominal conversion plan lost its declaration recipe")?;
            if (plan.source, plan.target) != (source, target)
                || plan.changed != schema.changed
                || plan.fields.len() != schema.fields.len()
            {
                return Err("nominal conversion plan changed its declaration/full application");
            }
        }
        Ok(plan)
    }
    pub(crate) fn retain_nominal_plan(
        &mut self,
        types: &Store,
        plan: Plan,
        pay: Payment<'_>,
    ) -> Result<(), &'static str> {
        if let Some(previous) = self.nominal_plan(types, plan.source, plan.target)? {
            return if *previous == plan {
                Ok(())
            } else {
                Err("nominal conversion plan changed its actual selected owner or layout")
            };
        }
        let plans = self
            .nominal
            .as_deref_mut()
            .ok_or("nominal conversion lacks its declaration recipes")?;
        let schema = plans
            .schemas
            .get(&plan.owner)
            .ok_or("nominal conversion lacks its selected declaration recipe")?;
        if plan.changed != schema.changed
            || plan.fields.len() != schema.fields.len()
            || plan.changed as usize >= plan.fields.len()
        {
            return Err("nominal conversion changed its selected declaration field schema");
        }
        let growth = Growth::map::<(TypeId, TypeId), Plan>(
            plans.transfers.len(),
            plans.transfers.capacity(),
            1,
        );
        plans.charge(
            growth.work.saturating_add(plan.fields.len()),
            growth.bytes.saturating_add(retained::vector(&plan.fields)),
        )?;
        pay(growth.work, growth.bytes)?;
        plans
            .transfers
            .try_reserve(1)
            .map_err(|_| "nominal conversion plan allocation failed")?;
        plans.transfers.insert((plan.source, plan.target), plan);
        Ok(())
    }
}
