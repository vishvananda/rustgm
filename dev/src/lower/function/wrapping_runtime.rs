//! A proved source Call uses signed modular byte indexing without reading pointee storage.
use super::{Lower, Selected, ir};
use crate::types::{Kind, instance::Instance};
impl Lower<'_, '_, '_> {
    pub(super) fn source_wrapping_offset(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source arith_offset call disappeared")?;
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
            .ok_or("source arith_offset callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selected) = self
            .context
            .constants
            .source_wrapping_offset(instance, self.body.source)?
        else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let Some(&[pointer, count]) = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source arith_offset requires two evaluated runtime inputs");
        };
        if signature != selected.signature()
            || value.ty != selected.pointer()
            || [pointer, count]
                .into_iter()
                .zip(selected.inputs())
                .any(|(id, ty)| {
                    self.body
                        .values
                        .get(id.0 as usize)
                        .is_none_or(|v| v.ty != ty)
                })
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual==self.body.callable_target(instance))
        {
            return Err("source arith_offset differs from its evaluated actual Call");
        }
        let pointer = self.value(pointer)?.runtime()?;
        let count = self.value(count)?.runtime()?;
        let size = self.context.layout(selected.element())?.size;
        let result =
            super::Context::wrapping_offset_value(&mut self.builder, pointer, count, size)?;
        Ok(Some(Selected::Runtime(result)))
    }
}
