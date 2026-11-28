//! Direct source intrinsic calls dispatch from their completed declarations.
use super::{Lower, Selected, ir};
use crate::{
    lowir::program::Operand,
    resolve::model::DefId,
    sema::compiler_items::{bit_counts, boxed_write, bytewise, swap_contract},
    support::intrinsics::Intrinsic,
    types::{GenericArgs, Kind, Primitive, TypeId, callable::Convention, instance::Instance},
};

impl Lower<'_, '_, '_> {
    pub(super) fn registered_intrinsic(
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
        let ty = self.input_type(callee)?;
        let Some(Kind::FunctionItem { item, .. }) = self.context.types.kind(ty) else {
            return Ok(None);
        };
        let Some(name) = self
            .context
            .constants
            .source_direct_intrinsic(DefId(item.0))?
        else {
            return Ok(None);
        };
        if let Some(lowered) =
            self.dispatch_registered_intrinsic(name, id, value, ty, signature, arguments)?
        {
            return Ok(Some(lowered));
        }
        let instance = self
            .body
            .callable_target(Instance::new(self.context.types, ty)?);
        let symbol = self.context.demand_trap(instance, signature, name)?;
        self.call_operand(signature, Operand::Symbol(symbol), arguments, value.ty)
            .map(Some)
    }
    fn dispatch_registered_intrinsic(
        &mut self,
        name: &'static str,
        id: ir::ValueId,
        value: &ir::Value,
        ty: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
    ) -> Result<Option<Selected>, &'static str> {
        match name {
            boxed_write::NAME => self.source_boxed_write(id, ty).map(Some),
            bytewise::NAME => self.source_raw_equality(id, ty).map(Some),
            swap_contract::NAME => self.source_typed_swap(id, ty).map(Some),
            "catch_unwind" => self.abort_profile_catch_unwind(id, ty).map(Some),
            "fabs"
            | "copysignf32"
            | "copysignf64"
            | "minimum_number_nsz_f32"
            | "minimum_number_nsz_f64"
            | "maximum_number_nsz_f32"
            | "maximum_number_nsz_f64" => self
                .source_float_sign(name, ty, signature, arguments, value.ty)
                .map(Some),
            "size_of_val" | "align_of_val" => self
                .pointed_value_layout(value, arguments, name == "size_of_val")
                .map(Some),
            "offset" => self
                .source_inbounds_offset(signature, arguments, value.ty)
                .map(Some),
            "slice_get_unchecked" => self
                .source_unchecked_index(ty, signature, arguments, value.ty)
                .map(Some),
            "ptr_offset_from" | "ptr_offset_from_unsigned" => self
                .source_pointer_distance(
                    signature,
                    arguments,
                    value.ty,
                    name == "ptr_offset_from_unsigned",
                )
                .map(Some),
            "compare_bytes" => self
                .source_compare_bytes(ty, signature, arguments, value.ty)
                .map(Some),
            "read_via_copy"
            | "write_via_move"
            | "volatile_load"
            | "volatile_store"
            | "unaligned_volatile_load"
            | "unaligned_volatile_store"
            | "forget" => self
                .source_memory_move(name, ty, signature, arguments, value.ty)
                .map(Some),
            "black_box" => self
                .source_black_box(ty, signature, arguments, value.ty)
                .map(Some),
            _ => {
                if let Some(lowered) =
                    self.source_float_math(name, ty, signature, arguments, value.ty)?
                {
                    return Ok(Some(lowered));
                }
                if bit_counts::named(name).is_some()
                    && let Some(lowered) =
                        self.source_bit_count(ty, signature, arguments, value.ty)?
                {
                    return Ok(Some(lowered));
                }
                if let Some(lowered) =
                    self.source_atomic_operation(name, ty, signature, arguments, value.ty)?
                {
                    return Ok(Some(lowered));
                }
                if let Some(transfer) = Intrinsic::source_transfer(name) {
                    return self
                        .source_counted_transfer(transfer, ty, signature, arguments, value.ty)
                        .map(Some);
                }
                self.unqualified_call(name, id)
            }
        }
    }
    fn source_compare_bytes(
        &mut self,
        callee: TypeId,
        signature: TypeId,
        arguments: ir::ValueList,
        result: TypeId,
    ) -> Result<Selected, &'static str> {
        let types = self.context.types;
        let Some(Kind::FunctionItem {
            arguments: GenericArgs::EMPTY,
            signature: declared,
            ..
        }) = types.kind(callee)
        else {
            return Err("source byte comparison requires its complete function item");
        };
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result: declared_result,
            unsafe_: true,
            convention: Convention::RUST,
        }) = types.kind(signature)
        else {
            return Err("source byte comparison requires its opened unsafe signature");
        };
        let Some(&[left, right, count]) = types.list(parameters) else {
            return Err("source byte comparison requires three inputs");
        };
        let inputs = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
            .ok_or("missing source byte comparison arguments")?;
        if declared != signature
            || result != declared_result
            || result != TypeId::primitive(Primitive::I32)
            || count != TypeId::primitive(Primitive::Usize)
            || left != right
            || !matches!(
                types.kind(left),
                Some(Kind::Pointer { mutable: false, pointee })
                    if pointee == TypeId::primitive(Primitive::U8)
            )
            || inputs.len() != 3
            || inputs.iter().zip([left, right, count]).any(|(&id, ty)| {
                self.body
                    .values
                    .get(id.0 as usize)
                    .is_none_or(|v| v.ty != ty)
            })
        {
            return Err("source byte comparison differs from its evaluated Call");
        }
        self.context
            .require_reached_body(self.body_index, self.body)?;
        let instance = self.body.callable_target(Instance::new(types, callee)?);
        let symbol = self.context.demand_compare_bytes(instance, signature)?;
        self.call_operand(signature, Operand::Symbol(symbol), arguments, result)
    }
}
