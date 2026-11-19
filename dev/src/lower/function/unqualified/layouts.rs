//! Sizes and alignments of pointed-to values, and the zero-validity
//! assertion when it holds for the asserted type.
use super::{
    super::{Address, Lower, Selected},
    Call,
};
use crate::{
    body::ir,
    lower::virtual_dispatch,
    lowir::vocabulary::{Binary, Type},
    types::{Kind, Primitive, TypeId, aggregate::ZeroFields, arguments::Argument},
};

impl Lower<'_, '_, '_> {
    pub(super) fn unqualified_layout(
        &mut self,
        call: Call<'_>,
    ) -> Result<Option<Selected>, &'static str> {
        match (call.name, call.inputs) {
            ("size_of_val" | "align_of_val", &[pointer]) => self.unqualified_extent(call, pointer),
            ("assert_zero_valid", &[]) => self.unqualified_assertion(call),
            _ => Ok(None),
        }
    }
    /// The size or alignment of a sized value, slice or trait object. A type
    /// with an unsized structural tail requires its enclosing layout.
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
        let size = call.name == "size_of_val";
        // A slice holds its length beside its address.
        let element = match types.kind(pointee) {
            Some(Kind::Slice(element)) => Some(element),
            Some(Kind::Primitive(Primitive::Str)) => Some(TypeId::primitive(Primitive::U8)),
            Some(Kind::Dynamic { .. }) => {
                return self
                    .unqualified_object_extent(pointer, pointee, size)
                    .map(Some);
            }
            Some(Kind::Foreign(_)) | None => return Ok(None),
            Some(_) if self.context.wide_referent(pointee)? => return Ok(None),
            Some(_) => None,
        };
        let layout = self.context.layout(element.unwrap_or(pointee))?;
        let fixed = if size {
            layout.size
        } else {
            u64::from(layout.align)
        };
        let mut extent = self.builder.integer(Type::I64, u128::from(fixed))?;
        if size && element.is_some() {
            let Selected::Object(address) = self.value(pointer)? else {
                return Err("slice pointer intrinsic input has no wide pointer storage");
            };
            let length = self.offset(address, 8)?;
            let length = self.builder.load(Type::I64, length.operand)?;
            extent = self.builder.binary(Binary::Mul, length, extent)?;
        }
        Ok(Some(Selected::Runtime(extent)))
    }

    fn unqualified_object_extent(
        &mut self,
        pointer: ir::ValueId,
        referent: TypeId,
        size: bool,
    ) -> Result<Selected, &'static str> {
        let Selected::Object(storage) = self.value(pointer)? else {
            return Err("trait object extent requires wide pointer storage");
        };
        let Some(table) = self.load_metadata(storage, referent)? else {
            return Err("trait object extent requires vtable metadata");
        };
        if table.ty != Type::Ptr {
            return Err("trait object extent has non-vtable metadata");
        }
        let offset = if size {
            virtual_dispatch::SIZE_OFFSET
        } else {
            virtual_dispatch::ALIGN_OFFSET
        };
        let entry = self.offset(
            Address {
                operand: table.operand,
                align: 8,
            },
            offset,
        )?;
        let value = self.builder.load(Type::I64, entry.operand)?;
        Ok(Selected::Runtime(value))
    }
    /// The assertion about the one type argument does nothing when it holds;
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
        let context = &mut *self.context;
        let valid = context.aggregates.zero_valid(types, ty, |ty| {
            if let Some(Kind::Pointer { pointee, .. }) = types.kind(ty) {
                return Ok(if context.wide_referent(pointee)? {
                    ZeroFields::Unproven
                } else {
                    ZeroFields::Valid
                });
            }
            context.layout(ty)?;
            Ok(context
                .layouts
                .zero_enum_fields(types, ty)?
                .map_or(ZeroFields::Unproven, ZeroFields::Visit))
        })?;
        Ok(valid.then_some(Selected::Unit))
    }
}
