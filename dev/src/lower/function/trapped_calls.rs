//! Bring-up only: a direct call of an unlowered source intrinsic calls a trap.
use super::{Lower, Selected, ir};
use crate::{
    lowir::program::Operand,
    resolve::model::DefId,
    types::{Kind, instance::Instance},
};

impl Lower<'_, '_, '_> {
    /// The caller keeps its body; only reaching this call stops the program.
    pub(super) fn trapped_call(
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
        let Some(name) = self.context.constants.source_trap(DefId(item.0))? else {
            return Ok(None);
        };
        let result = value.ty;
        let instance = self
            .body
            .callable_target(Instance::new(self.context.types, ty)?);
        let symbol = self.context.demand_trap(instance, signature, name)?;
        self.call_operand(signature, Operand::Symbol(symbol), arguments, result)
            .map(Some)
    }
}
