//! Closed public-layout recipes are data; selection and lifetime debts stay in bodies.
use super::{ItemId, Rules, Store, TypeId};
use crate::support::retained::{self, Growth, Payment};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Step {
    Slice(TypeId),
    Nested { source: TypeId, target: TypeId },
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
