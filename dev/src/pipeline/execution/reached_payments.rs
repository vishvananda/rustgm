//! A reached call prepares immutable code, then pays this caller's effects once.
use super::{Declarations, Engine, Error, Report, const_eval};
use crate::{
    resolve::unit::FileId,
    types::{Kind, instance::Instance},
};
impl Engine {
    fn function_for_reached(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        reached: &const_eval::ReachedConstCall,
    ) -> Result<bool, Error> {
        self.function_body(report, facts, reached.instance())
    }
    pub(super) fn prepare_reached_function(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        instance: Instance,
        state: Option<&mut const_eval::State>,
    ) -> Result<bool, Error> {
        let error = |e| Error::at(FileId(0), 0, e);
        crate::sema::compiler_items::check_const_drop_glue(facts, instance).map_err(error)?;
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
            return Err(error("source const target lost its function item"));
        };
        let definition = crate::resolve::model::DefId(item.0);
        let conditional = facts
            .source_bindings
            .as_deref()
            .map(|c| c.requires_const_payment(definition))
            .transpose()
            .map_err(error)?
            .unwrap_or(false);
        if !conditional {
            return self.source_layout(report, facts, instance, state.as_deref());
        }
        let state = state
            .ok_or_else(|| error("source conditional payment requires an actual reached caller"))?;
        let rows = self
            .const_calls
            .as_deref()
            .ok_or_else(|| error("source const-call inputs were not retained"))?;
        let limits = self.available_limits()?;
        let reached = state
            .reached_const_call(facts, &self.catalog, rows, instance, limits.steps)
            .map_err(error)?;
        if !self.function_for_reached(report, facts, &reached)? {
            return Ok(false);
        }
        let limits = self.available_limits()?;
        let rows = self
            .const_calls
            .as_deref()
            .ok_or_else(|| error("source const-call inputs were not retained"))?;
        state
            .pay_const_call(facts, &self.catalog, rows, &reached, limits)
            .map_err(error)?;
        Ok(true)
    }
}

#[cfg(test)]
impl super::Context {
    pub(crate) fn inspect_const_call<T>(
        &self,
        target: super::Target,
        use_inputs: impl FnOnce(&const_eval::code::Catalog, &const_eval::use_inputs::Inputs, usize) -> T,
    ) -> Result<T, &'static str> {
        let engine = self.engine.try_borrow().map_err(|_| "provider is active")?;
        let index = *engine
            .roots
            .get(&target)
            .ok_or("missing checked constant root")?;
        let rows = engine
            .const_calls
            .as_deref()
            .ok_or("missing checked const inputs")?;
        Ok(use_inputs(&engine.catalog, rows, index))
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/payment_storage.rs"]
mod storage_tests;
