//! Element-scaled pointer arithmetic and distances over thin data pointers.
use super::{
    super::{Context, Lower, Selected},
    Call,
};
use crate::{
    body::ir,
    lowir::{
        program::Value,
        vocabulary::{Binary, Type},
    },
    types::{Kind, Primitive, TypeId},
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_offset(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        let &[first, second] = call.inputs else {
            return Ok(None);
        };
        if matches!(call.name, "ptr_offset_from" | "ptr_offset_from_unsigned") {
            return self.unqualified_distance(call, first, second);
        }
        let Some(count) = self.integer_input(second)? else {
            return Ok(None);
        };
        let base = if call.name == "offset" {
            // The result is the pointer type itself, moved by whole pointees.
            if call.result != self.input_type(first)? {
                return Ok(None);
            }
            self.thin_input(first)?
        } else {
            self.slice_data(first, call.result)?
        };
        let Some((base, element)) = base else {
            return Ok(None);
        };
        if count.value.ty != Type::I64 {
            return Ok(None);
        }
        // A zero-sized element leaves the address unchanged.
        let size = self.context.layout(element)?.size;
        Context::wrapping_offset_value(&mut self.builder, base, count.value, size)
            .map(|value| Some(Selected::Runtime(value)))
    }
    /// The data address of a slice pointer whose element the thin result names.
    fn slice_data(
        &mut self,
        id: ir::ValueId,
        result: TypeId,
    ) -> Result<Option<(Value, TypeId)>, &'static str> {
        let referent = |kind: Option<Kind>| match kind {
            Some(Kind::Pointer { pointee, .. }) => Some(pointee),
            Some(Kind::Reference { referent, .. }) => Some(referent),
            _ => None,
        };
        let types = self.context.types;
        let Some(Kind::Slice(element)) =
            referent(types.kind(self.input_type(id)?)).and_then(|slice| types.kind(slice))
        else {
            return Ok(None);
        };
        if referent(types.kind(result)) != Some(element) || self.context.wide_referent(element)? {
            return Ok(None);
        }
        let Selected::Object(address) = self.value(id)? else {
            return Err("slice pointer intrinsic input has no wide pointer storage");
        };
        let data = self.builder.load(Type::Ptr, address.operand)?;
        Ok(Some((data, element)))
    }
    /// The byte difference divided by the pointee size, which divides it exactly.
    fn unqualified_distance(
        &mut self,
        call: Call<'_>,
        pointer: ir::ValueId,
        base: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let unsigned = call.name == "ptr_offset_from_unsigned";
        let word = if unsigned {
            Primitive::Usize
        } else {
            Primitive::Isize
        };
        let (Some((pointer, element)), Some((base, origin))) =
            (self.thin_input(pointer)?, self.thin_input(base)?)
        else {
            return Ok(None);
        };
        let size = self.context.layout(element)?.size;
        // The distance between zero-sized pointees is not defined.
        if element != origin || call.result != TypeId::primitive(word) || size == 0 {
            return Ok(None);
        }
        let pointer = self.builder.copy(Type::I64, pointer)?;
        let base = self.builder.copy(Type::I64, base)?;
        let mut distance = self.builder.binary(Binary::Sub, pointer, base)?;
        if size != 1 {
            let size = self.builder.integer(Type::I64, u128::from(size))?;
            let divide = if unsigned { Binary::Udiv } else { Binary::Div };
            distance = self.builder.binary(divide, distance, size)?;
        }
        Ok(Some(Selected::Runtime(distance)))
    }
}
