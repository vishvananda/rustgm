//! Reached source layout operations consume shared data and the actual evaluated call.
use super::{Lower, Selected, ir};
use crate::{
    lowir::{
        program::{Operand, Terminator},
        vocabulary::Type,
    },
    sema::compiler_items::RuntimeLayout,
    types::{Kind, instance::Instance},
};
impl Lower<'_, '_, '_> {
    fn layout_call(&mut self, id: ir::ValueId) -> Result<Option<RuntimeLayout>, &'static str> {
        if !self.context.constants.has_runtime_layouts() {
            return Ok(None);
        }
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("runtime layout lost its value")?;
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Ok(None);
        };
        let ty = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("runtime layout lost its callee")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let raw = Instance::new(self.context.types, ty)?;
        let target = self.body.callable_target(raw);
        let Some(plan) = self.context.constants.runtime_layout(target) else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        if signature != plan.signature
            || value.ty != plan.result
            || !self
                .body
                .arguments
                .get(arguments.start as usize..arguments.end as usize)
                .is_some_and(<[_]>::is_empty)
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual == target)
        {
            return Err("runtime layout differs from its actual evaluated Call/instance");
        }
        Ok(Some(plan))
    }
    pub(super) fn layout_panic(&mut self, id: ir::ValueId) -> Result<bool, &'static str> {
        let Some(plan) = self.layout_call(id)? else {
            return Ok(false);
        };
        let Some(assertion) = plan.assertion else {
            return Ok(false);
        };
        let Some(text) = assertion.message else {
            return Ok(false);
        };
        let symbol = self.context.demand(assertion.panic.instance)?;
        let representation = self.context.representation(assertion.panic.parameter)?;
        let message = self.text_literal(text, representation)?;
        self.call_arguments(
            assertion.panic.signature,
            Operand::Symbol(symbol),
            super::argument_sources::Arguments::Single {
                ty: assertion.panic.parameter,
                value: message,
            },
            crate::types::TypeId::NEVER,
        )?;
        self.builder.terminate(Terminator::Unreachable)?;
        Ok(true)
    }
    pub(super) fn layout_value(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let Some(plan) = self.layout_call(id)? else {
            return Ok(None);
        };
        if plan
            .assertion
            .is_some_and(|assertion| assertion.message.is_some())
        {
            return Err("runtime inhabited panic requires its reached statement");
        }
        if plan.result == crate::types::TypeId::UNIT {
            return Ok(Some(Selected::Unit));
        }
        Ok(Some(Selected::Runtime(
            self.builder.integer(Type::I64, u128::from(plan.bits))?,
        )))
    }
}
