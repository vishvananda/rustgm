//! Read prepared wrapper data without starting semantic selection or granting execution.
use super::{Reader, TypeId};
use crate::{
    resolve::unit::NodeKey,
    types::unsizing::{CoercionPlan, Conversion},
};

impl<'a> Reader<'a> {
    pub(crate) fn nominal_conversion(
        self,
        source: NodeKey,
        from: TypeId,
        to: TypeId,
    ) -> Result<&'a CoercionPlan, &'static str> {
        if !matches!(
            self.facts.unsizing_adjustments.get(&source),
            Some(Conversion::Nominal { .. })
        ) {
            return Err("nominal conversion lacks its actual checked source adjustment");
        }
        self.nominal_plan(from, to)
    }
    pub(crate) fn nominal_plan(
        self,
        source: TypeId,
        target: TypeId,
    ) -> Result<&'a CoercionPlan, &'static str> {
        let role = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or("nominal physical data lacks its source catalog")?
            .validated_role(crate::support::bindings::LangItem::CoerceUnsized)?
            .ok_or("nominal physical data lost its actual CoerceUnsized role")?;
        if crate::sema::compiler_items::unsizing_role(self.facts, role.definition)?
            != Some(crate::support::bindings::LangItem::CoerceUnsized)
        {
            return Err("nominal physical data changed its actual source role");
        }
        let plan = self
            .facts
            .unsizing_rules
            .as_deref()
            .ok_or("nominal conversion lacks its actual recipe owner")?
            .nominal_plan(&self.facts.types, source, target)?
            .ok_or("nominal conversion requires its checked wrapper conversion plan")?;
        if !self
            .facts
            .generics
            .implementations
            .as_ref()
            .is_some_and(|registry| {
                registry.is_validated(crate::resolve::model::DefId(plan.owner.0))
            })
        {
            return Err("nominal physical data lost its validated implementation owner");
        }
        Ok(plan)
    }
}
