//! Source metadata uses the proven result type and the actual evaluated pointer storage.
use super::{Lower, Selected, ir};
use crate::{
    abi::layout::Shape,
    lowir::vocabulary::Type,
    types::{Kind, Primitive, TypeId, instance::Instance},
};
impl Lower<'_, '_, '_> {
    pub(super) fn source_ptr_metadata(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source ptr_metadata call disappeared")?;
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
            .ok_or("source ptr_metadata callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selected) = self
            .context
            .constants
            .source_ptr_metadata(instance, self.body.source)?
        else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let [input] = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source ptr_metadata argument range disappeared")?
        else {
            return Err("source ptr_metadata requires one evaluated pointer argument");
        };
        let input = *input;
        let input_ty = self
            .context
            .constants
            .source_ptr_metadata_input(&selected)?;
        if signature != selected.signature()
            || value.ty != selected.output()
            || self
                .body
                .values
                .get(input.0 as usize)
                .is_none_or(|v| v.ty != input_ty)
            || !matches!(self.value(callee)?, Selected::Item(actual) if actual == self.body.callable_target(instance))
        {
            return Err("source ptr_metadata differs from its evaluated actual Call");
        }
        // The ordinary value cache evaluates this argument exactly once, including unit.
        let pointer = self.value(input)?;
        let shape = self.context.layout(input_ty)?.shape;
        let result = if selected.output() == TypeId::UNIT {
            if !matches!(shape, Shape::Scalar) {
                return Err("source ptr_metadata unit requires an actual thin pointer");
            }
            Selected::Unit
        } else if selected.output() == TypeId::primitive(Primitive::Usize) {
            if !matches!(shape, Shape::WidePointer) {
                return Err("source ptr_metadata length requires an actual wide pointer");
            }
            let Selected::Object(address) = pointer else {
                return Err("source ptr_metadata lost its wide pointer storage");
            };
            let word = self.offset(address, 8)?;
            Selected::Runtime(self.builder.load(Type::I64, word.operand)?)
        } else {
            return Err(
                "source ptr_metadata dynamic or unresolved result requires its real metadata storage producer",
            );
        };
        Ok(Some(result))
    }
}

impl Lower<'_, '_, '_> {
    pub(super) fn source_raw_parts(
        &mut self,
        id: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let value = self
            .body
            .values
            .get(id.0 as usize)
            .ok_or("source aggregate_raw_ptr value disappeared")?;
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
            .ok_or("source aggregate_raw_ptr callee disappeared")?
            .ty;
        if !matches!(self.context.types.kind(ty), Some(Kind::FunctionItem { .. })) {
            return Ok(None);
        }
        let instance = Instance::new(self.context.types, ty)?;
        let Some(selected) = self
            .context
            .constants
            .source_raw_parts(instance, self.body.source)?
        else {
            return Ok(None);
        };
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let [data, meta] = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("source aggregate_raw_ptr argument range disappeared")?
        else {
            return Err("source aggregate_raw_ptr requires exactly two evaluated arguments");
        };
        let (data, meta) = (*data, *meta);
        let inputs = self.context.constants.source_raw_parts_inputs(&selected)?;
        if signature != selected.signature()
            || value.ty != selected.output()
            || [data, meta].into_iter().zip(inputs).any(|(id, ty)| {
                self.body
                    .values
                    .get(id.0 as usize)
                    .is_none_or(|v| v.ty != ty)
            })
            || !matches!(self.value(callee)?,Selected::Item(actual)if actual==self.body.callable_target(instance))
        {
            return Err("source aggregate_raw_ptr differs from its evaluated actual Call");
        }
        let pointer = self.value(data)?.runtime()?;
        let metadata = self.value(meta)?;
        if !matches!(self.context.layout(inputs[0])?.shape, Shape::Scalar) {
            return Err("source aggregate_raw_ptr data requires its actual thin pointer ABI");
        }
        let output = self.context.layout(selected.output())?.shape;
        let mut kinds = [crate::abi::metadata::Metadata::Thin; 2];
        for (slot, ty) in kinds.iter_mut().zip([inputs[0], selected.output()]) {
            let Some(Kind::Pointer { pointee, .. }) = self.context.types.kind(ty) else {
                return Err("source aggregate_raw_ptr requires actual raw pointer representations");
            };
            *slot = self.context.layouts.pointer_metadata(
                self.context.types,
                self.context.aggregates,
                pointee,
            )?;
        }
        if kinds[0] != crate::abi::metadata::Metadata::Thin {
            return Err("source aggregate_raw_ptr data metadata must actually be thin");
        }
        let result = if inputs[1] == TypeId::UNIT
            && matches!(output, Shape::Scalar)
            && kinds[1] == crate::abi::metadata::Metadata::Thin
        {
            Selected::Runtime(pointer)
        } else if inputs[1] == TypeId::primitive(Primitive::Usize)
            && matches!(output, Shape::WidePointer)
            && kinds[1] == crate::abi::metadata::Metadata::Length
        {
            self.wide_pointer(pointer, metadata.runtime()?)?
        } else {
            return Err(
                "source aggregate_raw_ptr dynamic or unresolved metadata requires its actual value producer",
            );
        };
        Ok(Some(result))
    }
}
