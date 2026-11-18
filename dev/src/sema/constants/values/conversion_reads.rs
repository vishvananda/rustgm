//! Read prepared wrapper data without starting semantic selection or granting execution.
use super::{Reader, TypeId};
use crate::{
    resolve::unit::NodeKey,
    types::{
        Kind, RegionId,
        unsizing::{CoercionPlan, CoercionStep, Conversion},
    },
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
    /// Follow the checked changed fields to the physical pointer conversion.
    pub(crate) fn nominal_terminal(
        self,
        source: NodeKey,
        from: TypeId,
        to: TypeId,
    ) -> Result<&'a CoercionPlan, &'static str> {
        let mut plan = self.nominal_conversion(source, from, to)?;
        for _ in 0..1_000_000 {
            self.facts.charge_caller_inputs(1, 0)?;
            let CoercionStep::Nested { source, target } = plan.step else {
                return Ok(plan);
            };
            plan = self.nominal_plan(source, target)?;
        }
        Err("nested nominal conversion work budget exceeded")
    }

    /// Read the receiver lifetime used to select newly erased callable entries.
    pub(crate) fn nominal_object(
        self,
        source: NodeKey,
        from: TypeId,
        to: TypeId,
    ) -> Result<Option<(TypeId, TypeId, RegionId)>, &'static str> {
        let terminal = self.nominal_terminal(source, from, to)?;
        let CoercionStep::Object { concrete, object } = terminal.step else {
            return Ok(None);
        };
        let span = terminal
            .fields
            .get(terminal.changed as usize)
            .ok_or("nominal object conversion lost its changed field")?;
        let types = &self.facts.types;
        let pointer = match types.kind(span.source) {
            Some(Kind::Pattern { base, .. }) => types.kind(base),
            kind => kind,
        };
        let ((Some(Kind::Reference { region, .. }), _)
        | (Some(Kind::Pointer { .. }), Some(Kind::Dynamic { region, .. }))) =
            (pointer, types.kind(object))
        else {
            return Err("nominal object conversion lost its pointer lifetime");
        };
        Ok(Some((concrete, object, region)))
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
