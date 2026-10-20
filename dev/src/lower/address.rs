//! Address provenance and alignment shared by expression and cleanup lowering.
use crate::lowir::{
    builder::Builder,
    program::{Operand, Value},
    vocabulary::{Binary, Type},
};

#[derive(Clone, Copy)]
pub(super) struct Address {
    pub operand: Operand,
    pub align: u32,
}
impl Address {
    pub fn pointer(self, builder: &mut Builder) -> Result<Value, &'static str> {
        if matches!(
            self.operand,
            Operand::Slot(_) | Operand::Symbol(_) | Operand::Global(_)
        ) {
            builder.address(self.operand)
        } else {
            Ok(Value {
                ty: Type::Ptr,
                operand: self.operand,
            })
        }
    }
    pub fn offset(self, builder: &mut Builder, offset: u64) -> Result<Self, &'static str> {
        if offset == 0 {
            return Ok(self);
        }
        let pointer = self.pointer(builder)?;
        let value = builder.integer(Type::I64, u128::from(offset))?;
        let operand = builder.index(Type::U8, pointer, value)?.operand;
        let align = self.align.min(1_u32 << offset.trailing_zeros().min(29));
        Ok(Self { operand, align })
    }
    pub fn indexed(
        self,
        builder: &mut Builder,
        index: Value,
        stride: u64,
        align: u32,
    ) -> Result<Self, &'static str> {
        let stride = builder.integer(Type::I64, u128::from(stride))?;
        let offset = builder.binary(Binary::Mul, index, stride)?;
        let pointer = self.pointer(builder)?;
        let operand = builder.index(Type::U8, pointer, offset)?.operand;
        Ok(Self {
            operand,
            align: self.align.min(align),
        })
    }
}

impl super::context::Context<'_> {
    pub fn place_field(
        &mut self,
        ty: crate::types::TypeId,
        selection: crate::body::ir::Selection,
    ) -> Result<(crate::types::TypeId, crate::abi::tails::Offset), &'static str> {
        if self.wide_referent(ty)? {
            let (variant, field) = self.field_indices(selection)?;
            if variant != 0 {
                return Err("unsized place cannot have an enum variant");
            }
            return self
                .layouts
                .unsized_field(self.types, self.aggregates, ty, field)
                .map_err(|error| error.message);
        }
        let layout = self.layout(ty)?;
        self.selected_field(&layout, selection)
            .map(|(ty, offset)| (ty, crate::abi::tails::Offset::Fixed(offset)))
    }

    pub fn selected_field(
        &self,
        layout: &crate::abi::layout::Layout,
        selection: crate::body::ir::Selection,
    ) -> Result<(crate::types::TypeId, u64), &'static str> {
        let (variant, field) = self.field_indices(selection)?;
        let variant = self
            .layouts
            .variants(layout)
            .get(variant as usize)
            .ok_or("missing selected variant layout")?;
        let ty = *self
            .types
            .list(variant.fields)
            .and_then(|fields| fields.get(field as usize))
            .ok_or("missing selected field type")?;
        let offset = *self
            .layouts
            .offsets(variant)
            .get(field as usize)
            .ok_or("missing selected field offset")?;
        Ok((ty, offset))
    }
    fn field_indices(
        &self,
        selection: crate::body::ir::Selection,
    ) -> Result<(u32, u32), &'static str> {
        use crate::body::ir::Selection;
        Ok(match selection {
            Selection::Tuple(index) => (0, index),
            Selection::Nominal(id) => (
                self.members
                    .variant(id)
                    .and_then(|v| self.members.variants.get(v.0 as usize))
                    .ok_or("missing selected field variant")?
                    .ordinal,
                self.members
                    .ordinal(id)
                    .ok_or("missing selected field ordinal")?,
            ),
        })
    }
}
