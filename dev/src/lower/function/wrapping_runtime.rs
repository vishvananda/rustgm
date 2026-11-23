//! Authenticated source pointer arithmetic uses byte addresses without accessing pointees.
use super::{Lower, Selected, ir};
use crate::lowir::vocabulary::{Binary, Type};
use crate::types::{Kind, Primitive, TypeId, callable::Convention, instance::Instance};
impl Lower<'_, '_, '_> {
    pub(super) fn source_inbounds_offset(
        &mut self,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let [(pointer, pointer_type), (count, count_type)] =
            self.source_pointer_inputs(signature, arguments, result)?;
        let types = self.context.types;
        let Some(Kind::Pointer { pointee, .. }) = types.kind(pointer_type) else {
            return Err("source offset requires a raw pointer");
        };
        if result != pointer_type
            || !matches!(
                types.kind(count_type),
                Some(Kind::Primitive(Primitive::Isize | Primitive::Usize))
            )
            || self.context.wide_referent(pointee)?
        {
            return Err("source offset requires a thin pointer and a machine word");
        }
        let pointer = self.value(pointer)?.runtime()?;
        let count = self.value(count)?.runtime()?;
        let size = self.context.layout(pointee)?.size;
        // The unsafe call assumes an in-bounds offset. Byte indexing preserves
        // its starting pointer and scales either count by the pointee size.
        let result =
            super::Context::wrapping_offset_value(&mut self.builder, pointer, count, size)?;
        Ok(Selected::Runtime(result))
    }

    pub(super) fn source_pointer_distance(
        &mut self,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
        unsigned: bool,
    ) -> Result<Selected, &'static str> {
        let [(pointer, pointer_type), (base, base_type)] =
            self.source_pointer_inputs(signature, arguments, result)?;
        let Some(Kind::Pointer {
            mutable: false,
            pointee,
        }) = self.context.types.kind(pointer_type)
        else {
            return Err("source pointer distance requires a const raw pointer");
        };
        let word = if unsigned {
            Primitive::Usize
        } else {
            Primitive::Isize
        };
        if pointer_type != base_type
            || result != TypeId::primitive(word)
            || self.context.wide_referent(pointee)?
        {
            return Err("source pointer distance requires matching thin pointers and result");
        }
        let size = self.context.layout(pointee)?.size;
        if size == 0 {
            return Err("source pointer distance requires a nonzero pointee size");
        }
        let pointer = self.value(pointer)?.runtime()?;
        let base = self.value(base)?.runtime()?;
        let pointer = self.builder.copy(Type::I64, pointer)?;
        let base = self.builder.copy(Type::I64, base)?;
        let mut distance = self.builder.binary(Binary::Sub, pointer, base)?;
        // The unsafe call requires both addresses in the same allocation,
        // separated by a whole number of elements.
        if size != 1 {
            let size = self.builder.integer(Type::I64, u128::from(size))?;
            let divide = if unsigned { Binary::Udiv } else { Binary::Div };
            distance = self.builder.binary(divide, distance, size)?;
        }
        Ok(Selected::Runtime(distance))
    }

    fn source_pointer_inputs(
        &self,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<[(ir::ValueId, TypeId); 2], &'static str> {
        let Some(&[first, second]) = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source pointer arithmetic requires two evaluated inputs");
        };
        let first_type = self.input_type(first)?;
        let second_type = self.input_type(second)?;
        let types = self.context.types;
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result: declared_result,
            unsafe_: true,
            convention: Convention::RUST,
        }) = types.kind(signature)
        else {
            return Err("source pointer arithmetic requires its opened unsafe signature");
        };
        if result != declared_result || types.list(parameters) != Some(&[first_type, second_type]) {
            return Err("source pointer arithmetic differs from its checked signature");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        Ok([(first, first_type), (second, second_type)])
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
