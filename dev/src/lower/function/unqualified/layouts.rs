//! Sizes and alignments of pointed-to values, and validity assertions that
//! hold for the asserted type.
use super::{
    super::{Lower, Selected},
    Call,
};
use crate::{
    body::ir,
    lowir::vocabulary::{Binary, Type},
    types::{Kind, Primitive, TypeId, arguments::Argument},
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_layout(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        match (call.name, call.inputs) {
            ("size_of_val" | "align_of_val", &[pointer]) => self.unqualified_extent(call, pointer),
            ("assert_inhabited" | "assert_zero_valid", &[]) => self.unqualified_assertion(call),
            _ => Ok(None),
        }
    }
    /// The size or alignment of a sized value, a slice or a string slice. A
    /// trait object or a type with an unsized tail keeps the trap.
    fn unqualified_extent(
        &mut self,
        call: Call<'_>,
        pointer: ir::ValueId,
    ) -> Result<Option<Selected>, &'static str> {
        let types = self.context.types;
        let Some(Kind::Pointer { pointee, .. }) = types.kind(self.input_type(pointer)?) else {
            return Ok(None);
        };
        if call.result != TypeId::primitive(Primitive::Usize) {
            return Ok(None);
        }
        // A slice holds its length beside its address.
        let element = match types.kind(pointee) {
            Some(Kind::Slice(element)) => Some(element),
            Some(Kind::Primitive(Primitive::Str)) => Some(TypeId::primitive(Primitive::U8)),
            Some(Kind::Dynamic { .. } | Kind::Foreign(_)) | None => return Ok(None),
            Some(_) if self.context.wide_referent(pointee)? => return Ok(None),
            Some(_) => None,
        };
        let layout = self.context.layout(element.unwrap_or(pointee))?;
        let sized = call.name == "size_of_val";
        let fixed = if sized {
            layout.size
        } else {
            u64::from(layout.align)
        };
        let mut extent = self.builder.integer(Type::I64, u128::from(fixed))?;
        if sized && element.is_some() {
            let Selected::Object(address) = self.value(pointer)? else {
                return Err("slice pointer intrinsic input has no wide pointer storage");
            };
            let length = self.offset(address, 8)?;
            let length = self.builder.load(Type::I64, length.operand)?;
            extent = self.builder.binary(Binary::Mul, length, extent)?;
        }
        Ok(Some(Selected::Runtime(extent)))
    }
    /// An assertion about the one type argument does nothing when it holds;
    /// one that fails, or that this module cannot decide, keeps the trap.
    fn unqualified_assertion(&mut self, call: Call<'_>) -> Result<Option<Selected>, &'static str> {
        let types = self.context.types;
        let Some(Kind::FunctionItem { arguments, .. }) = types.kind(call.callee) else {
            return Ok(None);
        };
        let Some(mut arguments) = types.arguments(arguments) else {
            return Ok(None);
        };
        let (Some(Argument::Type(ty)), None) = (arguments.next(), arguments.next()) else {
            return Ok(None);
        };
        if call.result != TypeId::UNIT {
            return Ok(None);
        }
        let holds = if call.name == "assert_inhabited" {
            self.context.layout(ty)?.inhabited
        } else {
            self.zero_valid(ty)?
        };
        Ok(holds.then_some(Selected::Unit))
    }
    /// Whether all-zero bytes are a value of a type built from numbers,
    /// booleans, characters, thin raw pointers, arrays and tuples. Any other
    /// type is left undecided.
    fn zero_valid(&mut self, ty: TypeId) -> Result<bool, &'static str> {
        let types = self.context.types;
        let mut pending = vec![ty];
        let mut visited = 0_u32;
        while let Some(ty) = pending.pop() {
            visited += 1;
            match types.kind(ty) {
                Some(Kind::Primitive(primitive)) if primitive != Primitive::Str => {}
                Some(Kind::Pointer { pointee, .. }) if !self.context.wide_referent(pointee)? => {}
                Some(Kind::Array { element, .. }) => pending.push(element),
                Some(Kind::Tuple(fields)) if visited < 1024 => pending.extend_from_slice(
                    types
                        .list(fields)
                        .ok_or("asserted tuple type lost its fields")?,
                ),
                _ => return Ok(false),
            }
        }
        Ok(true)
    }
}
