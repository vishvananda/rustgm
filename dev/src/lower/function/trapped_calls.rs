//! Bodyless source intrinsics dispatch from their completed declarations.
use super::{Lower, Selected, ir};
use crate::{
    lowir::program::Operand,
    resolve::model::DefId,
    types::{Kind, instance::Instance},
};

impl Lower<'_, '_, '_> {
    pub(super) fn bodiless_intrinsic(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("trapped call disappeared")?;
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
            .ok_or("trapped callee disappeared")?
            .ty;
        let Some(Kind::FunctionItem { item, .. }) = self.context.types.kind(ty) else {
            return Ok(None);
        };
        let Some(name) = self
            .context
            .constants
            .source_bodiless_intrinsic(DefId(item.0))?
        else {
            return Ok(None);
        };
        if name == "catch_unwind" {
            return self.abort_profile_catch_unwind(id, ty).map(Some);
        }
        if matches!(name, "size_of_val" | "align_of_val") {
            return self
                .pointed_value_layout(value, arguments, name == "size_of_val")
                .map(Some);
        }
        if matches!(name, "bswap" | "bitreverse") {
            return self.bit_permutation(name, arguments, value.ty).map(Some);
        }
        if name == "offset" {
            return self
                .source_inbounds_offset(signature, arguments, value.ty)
                .map(Some);
        }
        if let Some(lowered) = self.unqualified_call(name, id)? {
            return Ok(Some(lowered));
        }
        let result = value.ty;
        let instance = self
            .body
            .callable_target(Instance::new(self.context.types, ty)?);
        let symbol = self.context.demand_trap(instance, signature, name)?;
        self.call_operand(signature, Operand::Symbol(symbol), arguments, result)
            .map(Some)
    }
}
