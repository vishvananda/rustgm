//! The actual checked source Call selects a boolean in this generation session.
use super::{Lower, Selected, ir};
use crate::{
    lowir::vocabulary::Type,
    support::runtime_checks::{RuntimeCheckBinding, RuntimeCheckConfig},
    types::{Kind, Primitive, TypeId, instance::Instance},
};

/// This borrowed operation cannot grant another body/call a runtime decision.
struct Call<'body> {
    body: &'body ir::Body,
    value: ir::ValueId,
    instance: Instance,
    signature: TypeId,
    binding: RuntimeCheckBinding,
    generation: RuntimeCheckConfig,
}

impl Lower<'_, '_, '_> {
    pub(super) fn runtime_ub_check(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source ub_checks call disappeared")?;
        let ir::Rvalue::Call {
            signature,
            callee,
            arguments,
        } = value.kind
        else {
            return Err("source ub_checks requires its actual Call");
        };
        let ty = self
            .body
            .values
            .get(callee.0 as usize)
            .ok_or("source ub_checks callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some((selection, binding, generation)) = self
            .context
            .constants
            .source_runtime_check(instance, self.body.source)?
        else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        if signature != selection.signature
            || value.ty != TypeId::primitive(Primitive::Bool)
            || !self
                .body
                .arguments
                .get(arguments.start as usize..arguments.end as usize)
                .is_some_and(<[ir::ValueId]>::is_empty)
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual == self.body.callable_target(instance))
        {
            return Err(
                "source ub_checks differs from its evaluated exact call/signature/arguments",
            );
        }
        let call = Call {
            body: self.body,
            value: id,
            instance,
            signature,
            binding,
            generation,
        };
        self.consume_runtime_check(call).map(Some)
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "consume the exact non-clonable body/call witness once"
    )]
    fn consume_runtime_check(&mut self, call: Call<'_>) -> Result<Selected, &'static str> {
        let actual = self
            .body
            .values
            .get(call.value.0 as usize)
            .ok_or("runtime check lost its exact Call")?;
        let ir::Rvalue::Call {
            signature, callee, ..
        } = actual.kind
        else {
            return Err("runtime check changed its actual operation");
        };
        if !std::ptr::eq(call.body, self.body)
            || signature != call.signature
            || self
                .body
                .values
                .get(callee.0 as usize)
                .is_none_or(|v| v.ty != call.instance.ty())
        {
            return Err("runtime check belongs to another actual body/call");
        }
        self.builder
            .integer(Type::I1, u128::from(call.binding.resolve(call.generation)))
            .map(Selected::Runtime)
    }
}
