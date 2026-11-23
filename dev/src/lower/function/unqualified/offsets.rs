//! Element indexing from the data address of the unqualified slice operation.
use super::{
    super::{Context, Lower, Selected},
    Call,
};
use crate::{
    body::ir,
    lowir::{program::Value, vocabulary::Type},
    types::{Kind, TypeId},
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_offset(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        let &[first, second] = call.inputs else {
            return Ok(None);
        };
        let Some(count) = self.integer_input(second)? else {
            return Ok(None);
        };
        let Some((base, element)) = self.slice_data(first, call.result)? else {
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
}
