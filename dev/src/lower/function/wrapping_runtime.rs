//! A proved source Call uses signed modular byte indexing without reading pointee storage.
use super::{Lower, Selected, ir};
use crate::types::{Kind, Primitive, TypeId, callable::Convention, instance::Instance};
impl Lower<'_, '_, '_> {
    pub(super) fn source_inbounds_offset(
        &mut self,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let Some(&[pointer, count]) = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source offset requires two evaluated inputs");
        };
        let pointer_type = self.input_type(pointer)?;
        let count_type = self.input_type(count)?;
        let types = self.context.types;
        let Some(Kind::Pointer { pointee, .. }) = types.kind(pointer_type) else {
            return Err("source offset requires a raw pointer");
        };
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result: declared_result,
            unsafe_: true,
            convention: Convention::RUST,
        }) = types.kind(signature)
        else {
            return Err("source offset requires its opened unsafe signature");
        };
        if result != pointer_type
            || result != declared_result
            || types.list(parameters) != Some(&[pointer_type, count_type])
            || !matches!(
                types.kind(count_type),
                Some(Kind::Primitive(Primitive::Isize | Primitive::Usize))
            )
            || self.context.wide_referent(pointee)?
        {
            return Err("source offset requires a thin pointer and a machine word");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let pointer = self.value(pointer)?.runtime()?;
        let count = self.value(count)?.runtime()?;
        let size = self.context.layout(pointee)?.size;
        // The unsafe call assumes an in-bounds offset. Byte indexing preserves
        // its starting pointer and scales either count by the pointee size.
        let result =
            super::Context::wrapping_offset_value(&mut self.builder, pointer, count, size)?;
        Ok(Selected::Runtime(result))
    }

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
        if !selected.opened_signature_matches(self.context.types, signature)
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
