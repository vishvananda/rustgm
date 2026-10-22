//! Opaque source identity retains its separately checked physical representation.
use super::Reader;
use crate::{
    abi::layout::{Layout, Shape},
    types::{Kind, TypeId},
};
#[cfg(test)]
#[path = "../../../../tests/invariants/head_layouts.rs"]
mod tests;

impl Reader<'_> {
    /// This selects physical storage only, never source equality or permission.
    pub(crate) fn storage_type(self, ty: TypeId) -> Result<TypeId, &'static str> {
        if !matches!(self.facts.types.kind(ty), Some(Kind::Opaque(..))) {
            return Ok(ty);
        }
        let head = self.facts.types.storage_head(ty)?;
        if matches!(self.facts.types.kind(head), None | Some(Kind::Opaque(..))) {
            return Err("opaque storage requires a canonical non-opaque defining head");
        }
        let logical = self
            .facts
            .layouts
            .get(ty)
            .ok_or("opaque storage requires its completed logical layout")?;
        let physical = self
            .facts
            .layouts
            .get(head)
            .ok_or("opaque storage requires its completed defining layout")?;
        if !same_layout(logical, physical) {
            return Err("opaque storage differs from its exact defining layout");
        }
        Ok(head)
    }
}

fn same_layout(left: &Layout, right: &Layout) -> bool {
    (
        left.size,
        left.align,
        left.inhabited,
        left.contains_reference,
        left.contains_explicit_align,
    ) == (
        right.size,
        right.align,
        right.inhabited,
        right.contains_reference,
        right.contains_explicit_align,
    ) && same_shape(left.shape, right.shape)
}
fn same_shape(left: Shape, right: Shape) -> bool {
    match (left, right) {
        (Shape::Scalar, Shape::Scalar) | (Shape::WidePointer, Shape::WidePointer) => true,
        (
            Shape::Array {
                element: left_element,
                stride: left_stride,
                length: left_length,
            },
            Shape::Array {
                element: right_element,
                stride: right_stride,
                length: right_length,
            },
        ) => {
            (left_element, left_stride, left_length) == (right_element, right_stride, right_length)
        }
        (
            Shape::Aggregate {
                variants: left_variants,
                tag: left_tag,
            },
            Shape::Aggregate {
                variants: right_variants,
                tag: right_tag,
            },
        ) => {
            (left_variants.start, left_variants.end) == (right_variants.start, right_variants.end)
                && left_tag.map(|tag| (tag.ty, tag.offset))
                    == right_tag.map(|tag| (tag.ty, tag.offset))
        }
        (
            Shape::Niche {
                variants: left_variants,
                encoding: left_encoding,
            },
            Shape::Niche {
                variants: right_variants,
                encoding: right_encoding,
            },
        ) => {
            (left_variants.start, left_variants.end, left_encoding)
                == (right_variants.start, right_variants.end, right_encoding)
        }
        _ => false,
    }
}
