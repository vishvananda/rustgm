//! The bring-up implementation of fixed-size raw equality calls one shim per instance.
use super::{
    super::{Lower, Selected},
    Call,
};
use crate::{
    lowir::program::Operand,
    types::{Primitive, TypeId, instance::Instance},
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_raw_eq(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        let &[left, right] = call.inputs else {
            return Ok(None);
        };
        if call.result != TypeId::primitive(Primitive::Bool) {
            return Ok(None);
        }
        let (Some((_, element)), Some((_, other))) =
            (self.thin_input(left)?, self.thin_input(right)?)
        else {
            return Ok(None);
        };
        if element != other {
            return Ok(None);
        }
        let instance = self
            .body
            .callable_target(Instance::new(self.context.types, call.callee)?);
        let symbol = self
            .context
            .demand_unqualified_raw_eq(instance, call.signature, element)?;
        self.call_operand(
            call.signature,
            Operand::Symbol(symbol),
            call.arguments,
            call.result,
        )
        .map(Some)
    }
}
