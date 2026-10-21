//! Effectful volatile operations remain explicit even when their result is unused.
use super::{Lower, Selected};
use crate::{
    body::ir::ValueList, lowir::vocabulary::Type, support::intrinsics::Intrinsic,
    types::instance::Instance,
};
impl Lower<'_, '_> {
    pub(super) fn intrinsic(
        &mut self,
        intrinsic: Intrinsic,
        instance: Instance,
        arguments: ValueList,
    ) -> Result<Selected, &'static str> {
        if !intrinsic.is_volatile() {
            let Some(crate::types::Kind::FunctionItem { signature, .. }) =
                self.context.types.kind(instance.ty())
            else {
                return Err("atomic intrinsic requires a selected callable");
            };
            let Some(crate::types::Kind::Function { result, .. }) =
                self.context.types.kind(signature)
            else {
                return Err("atomic intrinsic has no callable signature");
            };
            let symbol = self.context.demand_intrinsic(intrinsic, instance)?;
            return self.call_operand(
                signature,
                crate::lowir::program::Operand::Symbol(symbol),
                arguments,
                result,
            );
        }
        let super::Representation::Scalar(ty) = self.context.intrinsic_type(intrinsic, instance)?
        else {
            return Err("volatile operation requires a scalar representation");
        };
        let inputs = self.body.arguments(arguments);
        let pointer = self
            .value(*inputs.first().ok_or("missing volatile pointer operand")?)?
            .runtime()?;
        if pointer.ty != Type::Ptr {
            return Err("volatile operation requires a thin pointer value");
        }
        match intrinsic {
            Intrinsic::ReadVolatile if inputs.len() == 1 => self
                .builder
                .volatile_load(ty, pointer.operand)
                .map(Selected::Runtime),
            Intrinsic::WriteVolatile if inputs.len() == 2 => {
                let value = self.value(inputs[1])?.runtime()?;
                if value.ty != ty {
                    return Err("volatile store value differs from its pointer's pointee type");
                }
                self.builder.volatile_store(value, pointer.operand)?;
                Ok(Selected::Unit)
            }
            _ => Err("volatile operation argument count mismatch"),
        }
    }
}
