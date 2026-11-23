//! Bodyless source intrinsics dispatch from their completed declarations.
use super::{Lower, Selected, ir};
use crate::{
    lowir::program::Operand,
    resolve::model::DefId,
    sema::compiler_items::bit_counts,
    support::{intrinsics::Intrinsic, word_operation::Update},
    types::{GenericArgs, Kind, Primitive, TypeId, callable::Convention, instance::Instance},
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
        let ty = self.input_type(callee)?;
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
        if bit_counts::named(name).is_some()
            && let Some(lowered) = self.source_bit_count(ty, signature, arguments, value.ty)?
        {
            return Ok(Some(lowered));
        }
        if name == "offset" {
            return self
                .source_inbounds_offset(signature, arguments, value.ty)
                .map(Some);
        }
        if name == "slice_get_unchecked" {
            return self
                .source_unchecked_index(ty, signature, arguments, value.ty)
                .map(Some);
        }
        if matches!(name, "ptr_offset_from" | "ptr_offset_from_unsigned") {
            return self
                .source_pointer_distance(
                    signature,
                    arguments,
                    value.ty,
                    name == "ptr_offset_from_unsigned",
                )
                .map(Some);
        }
        if name == "compare_bytes" {
            return self
                .source_compare_bytes(ty, signature, arguments, value.ty)
                .map(Some);
        }
        if matches!(name, "atomic_load" | "atomic_store")
            && let Some(lowered) =
                self.source_atomic_access(name, ty, signature, arguments, value.ty)?
        {
            return Ok(Some(lowered));
        }
        if let Some(operation) = Update::intrinsic(name)
            && let Some(lowered) =
                self.source_atomic_update(operation, ty, signature, arguments, value.ty)?
        {
            return Ok(Some(lowered));
        }
        if matches!(name, "read_via_copy" | "write_via_move" | "forget") {
            return self
                .source_memory_move(name, ty, signature, arguments, value.ty)
                .map(Some);
        }
        if let Some(transfer) = Intrinsic::source_transfer(name) {
            return self
                .source_counted_transfer(transfer, ty, signature, arguments, value.ty)
                .map(Some);
        }
        if name == "black_box" {
            return self
                .source_black_box(ty, signature, arguments, value.ty)
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
