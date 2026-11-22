//! Runtime size and alignment come from the pointed-to storage and its metadata.
use super::{Lower, Selected, ir};
use crate::{
    abi::tails::{Alignment, Extent, Offset},
    lower::virtual_dispatch::{ALIGN_OFFSET, SIZE_OFFSET},
    lowir::{
        program::Value,
        vocabulary::{Binary, Type},
    },
    types::{Kind, Primitive, TypeId},
};

impl Lower<'_, '_, '_> {
    pub(super) fn pointed_value_layout(
        &mut self,
        value: &ir::Value,
        arguments: ir::ValueList,
        size: bool,
    ) -> Result<Selected, &'static str> {
        let Some(&[pointer]) = self
            .body
            .arguments
            .get(arguments.start as usize..arguments.end as usize)
        else {
            return Err("source value layout requires one pointer argument");
        };
        let input = self
            .body
            .values
            .get(pointer.0 as usize)
            .ok_or("source value layout lost its input")?;
        let types = self.context.types;
        let Some(Kind::Pointer {
            pointee: referent,
            mutable: false,
        }) = types.kind(input.ty)
        else {
            return Err("source value layout requires a const raw pointer");
        };
        if value.ty != TypeId::primitive(Primitive::Usize) {
            return Err("source value layout requires a usize result");
        }
        let referent = types.storage_head(referent)?;
        let extent = if self.context.wide_referent(referent)? {
            self.wide_value_layout(pointer, referent, size)?
        } else {
            let layout = self.context.layout(referent)?;
            let word = if size {
                layout.size
            } else {
                u64::from(layout.align)
            };
            self.builder.integer(Type::I64, u128::from(word))?
        };
        Ok(Selected::Runtime(extent))
    }

    fn wide_value_layout(
        &mut self,
        pointer: ir::ValueId,
        ty: TypeId,
        size: bool,
    ) -> Result<Value, &'static str> {
        let extent = self.unsized_extent(ty)?;
        if !size && !extent.alignment.is_dynamic() {
            return self.value_alignment(extent.alignment, None);
        }
        let Selected::Object(storage) = self.value(pointer)? else {
            return Err("unsized value layout requires wide pointer storage");
        };
        let metadata = self
            .load_metadata(storage, ty)?
            .ok_or("unsized value layout requires pointer metadata")?;
        let object_align = (metadata.ty == Type::Ptr && (!size || extent.tail.is_some()))
            .then(|| self.vtable_layout_word(metadata, ALIGN_OFFSET))
            .transpose()?;
        if size {
            self.unsized_value_size(ty, extent, metadata, object_align)
        } else {
            self.value_alignment(extent.alignment, object_align)
        }
    }

    fn unsized_extent(&mut self, ty: TypeId) -> Result<Extent, &'static str> {
        self.context
            .layouts
            .unsized_extent(self.context.types, self.context.aggregates, ty)
    }

    fn unsized_value_size(
        &mut self,
        mut ty: TypeId,
        mut extent: Extent,
        metadata: Value,
        object_align: Option<Value>,
    ) -> Result<Value, &'static str> {
        let mut layers = Vec::new();
        while let Some((tail, offset)) = extent.tail {
            let inner = self.unsized_extent(tail)?;
            if offset != Offset::Fixed(0) || extent.alignment != inner.alignment {
                layers.push((offset, extent.alignment));
            }
            ty = tail;
            extent = inner;
        }
        let mut bytes = self.unsized_leaf_size(ty, metadata)?;
        for (offset, alignment) in layers.into_iter().rev() {
            bytes = self.extend_tail_size(bytes, offset, alignment, object_align)?;
        }
        Ok(bytes)
    }

    fn unsized_leaf_size(&mut self, ty: TypeId, metadata: Value) -> Result<Value, &'static str> {
        match self.context.types.kind(ty) {
            Some(Kind::Primitive(Primitive::Str)) if metadata.ty == Type::I64 => Ok(metadata),
            Some(Kind::Slice(element)) => {
                let bytes = self.context.layout(element)?.size;
                let width = self.builder.integer(Type::I64, u128::from(bytes))?;
                self.builder.binary(Binary::Mul, metadata, width)
            }
            Some(Kind::Dynamic { .. }) => self.vtable_layout_word(metadata, SIZE_OFFSET),
            _ => Err("unsized value layout lost its concrete storage tail"),
        }
    }

    fn extend_tail_size(
        &mut self,
        mut size: Value,
        offset: Offset,
        alignment: Alignment,
        object_align: Option<Value>,
    ) -> Result<Value, &'static str> {
        let prefix = match offset {
            Offset::Fixed(0) => None,
            Offset::Fixed(bytes) => Some(self.builder.integer(Type::I64, u128::from(bytes))?),
            Offset::Aligned { prefix, alignment } => Some(self.align_tail_offset(
                prefix,
                alignment,
                object_align.ok_or("unsized tail offset requires object alignment")?,
            )?),
        };
        if let Some(prefix) = prefix {
            size = self.builder.binary(Binary::Add, size, prefix)?;
        }
        if alignment.maximum == 1 {
            return Ok(size);
        }
        let align = self.value_alignment(alignment, object_align)?;
        self.round_extent(size, align)
    }

    fn value_alignment(
        &mut self,
        alignment: Alignment,
        object_align: Option<Value>,
    ) -> Result<Value, &'static str> {
        if alignment.is_dynamic() {
            self.clamp_alignment(
                object_align.ok_or("dynamic extent requires object alignment")?,
                alignment,
            )
        } else {
            self.builder
                .integer(Type::I64, u128::from(alignment.minimum))
        }
    }
}
