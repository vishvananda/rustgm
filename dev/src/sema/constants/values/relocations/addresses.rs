//! Word equality follows retained storage ancestry, independently of a source view.
use super::super::{
    Constant, Evaluator, Failure, LocationId, Origin, Reader, Value, WritePermission,
};
use crate::types::TypeId;

/// No caller can construct a root/offset pointer: this records existing ancestry only.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct WordAddress {
    pub(super) root: LocationId,
    pub(super) offset: u64,
    pub(super) write: WritePermission,
}
impl WordAddress {
    pub(crate) const fn root(self) -> LocationId {
        self.root
    }
    pub(crate) const fn offset(self) -> u64 {
        self.offset
    }
}
impl Evaluator<'_> {
    /// A word at an array's end may still select an empty reference range.
    pub(crate) fn word_sequence_range(
        &mut self,
        address: WordAddress,
        length: u64,
        target: TypeId,
    ) -> Result<Constant, Failure> {
        let reader = Reader::new(self.facts());
        let root = reader.location(address.root)?.ty;
        let super::super::StoredShape::Array {
            element,
            stride,
            length: bound,
        } = reader.storage_shape(root)?
        else {
            return Err("symbolic sequence word requires original array storage".into());
        };
        if !reader.same_storage_type(element, reader.sequence_element(target)?) {
            return Err("symbolic sequence word has a different stored element".into());
        }
        let from = if stride == 0 && address.offset == 0 {
            0
        } else if stride != 0 && address.offset.is_multiple_of(stride) {
            address.offset / stride
        } else {
            return Err("symbolic sequence word is not at an element boundary".into());
        };
        if from.checked_add(length).is_none_or(|end| end > bound) {
            return Err(Failure::Fault("constant range is out of bounds"));
        }
        let mut pointer = super::super::references::Pointer::new(address.root.0);
        pointer.write = address.write;
        self.subsequence(Constant(Value::Reference(pointer)), from, length, target)
    }
    pub(crate) fn word_address(
        &mut self,
        value: Constant,
        limit: usize,
    ) -> Result<(WordAddress, usize), Failure> {
        let mut work = 0;
        let address = self.word_address_charged(value, limit, &mut work)?;
        Ok((address, work))
    }
    pub(super) fn word_address_charged(
        &mut self,
        value: Constant,
        limit: usize,
        work: &mut usize,
    ) -> Result<WordAddress, Failure> {
        let Value::Reference(pointer) = value.0 else {
            return Err("symbolic data word requires existing pointer provenance".into());
        };
        if pointer.metadata.is_some() {
            return Err("symbolic data word must separate pointer metadata".into());
        }
        let mut root = LocationId(pointer.place);
        let mut offset = 0u64;
        loop {
            *work = work.saturating_add(1);
            if *work > limit {
                return Err("symbolic address ancestry work budget exceeded".into());
            }
            let place = Reader::new(self.facts()).location(root)?;
            if let Origin::ByteOffset {
                base,
                offset: displacement,
            } = place.origin
            {
                offset = offset
                    .checked_add(displacement)
                    .ok_or("symbolic byte address offset overflow")?;
                root = base;
                continue;
            }
            let (base, index) = match place.origin {
                Origin::Projection { base, index } => (base, index),
                Origin::Subsequence { base, from, .. } => (base, from),
                Origin::Allocation | Origin::Static(_) => break,
                Origin::ByteOffset { .. } => return Err("symbolic byte ancestry changed".into()),
            };
            let parent = Reader::new(self.facts()).location(base)?;
            let (_, added) = self.storage_layout(parent.ty, limit.saturating_sub(*work))?;
            *work = work
                .checked_add(added)
                .filter(|n| *n <= limit)
                .ok_or("symbolic address layout work budget exceeded")?;
            let reader = Reader::new(self.facts());
            let displacement = match place.origin {
                Origin::Projection { .. } => {
                    let field = reader.projection_layout(parent.ty, index)?;
                    if !reader.same_storage_type(field.ty, place.ty) {
                        return Err(
                            "symbolic address projection has a different stored type".into()
                        );
                    }
                    field.offset
                }
                Origin::Subsequence { length, .. } => {
                    reader.sequence_displacement(parent.ty, index, length, place.ty)?
                }
                _ => return Err("symbolic address ancestry changed".into()),
            };
            offset = offset
                .checked_add(displacement)
                .ok_or("symbolic address offset overflow")?;
            root = base;
        }
        Ok(WordAddress {
            root,
            offset,
            write: pointer.write,
        })
    }
    /// Select bytes relative to existing symbolic provenance, never a numeric address.
    pub(crate) fn byte_projection(
        &mut self,
        value: Constant,
        offset: u64,
        ty: TypeId,
        limit: usize,
    ) -> Result<(Constant, usize), Failure> {
        let Value::Reference(mut pointer) = value.0 else {
            return Err("constant byte projection lacks pointer provenance".into());
        };
        if self.facts().types.is_sized(ty) != Some(true) {
            return Err("constant byte projection requires a completed sized target".into());
        }
        pointer.metadata = None;
        let (address, mut work) = self.word_address(Constant(Value::Reference(pointer)), limit)?;
        let root = Reader::new(self.facts()).location(address.root)?.ty;
        let (layout, added) = self.storage_layout(root, limit.saturating_sub(work))?;
        work = work
            .checked_add(added)
            .filter(|n| *n <= limit)
            .ok_or("constant byte address work budget exceeded")?;
        let offset = address
            .offset
            .checked_add(offset)
            .filter(|&n| n <= layout.size && i64::try_from(n).is_ok())
            .ok_or("constant byte address is outside allocation bounds")?;
        pointer.place = address.root.0;
        let value = self
            .queries
            .typing
            .facts
            .constant_references
            .byte_projection(
                pointer,
                offset,
                super::super::references::Place {
                    value: None,
                    ty,
                    access: super::super::references::Access::Owned,
                    packed: None,
                    projection: None,
                },
            )?;
        Ok((Constant(value), work))
    }
    /// A canonical word supplies provenance and restrictions, never numeric permission.
    pub(crate) fn byte_word_projection(
        &mut self,
        address: WordAddress,
        ty: TypeId,
        limit: usize,
    ) -> Result<(Constant, usize), Failure> {
        let mut pointer = super::super::references::Pointer::new(address.root.0);
        pointer.write = address.write;
        self.byte_projection(
            Constant(Value::Reference(pointer)),
            address.offset,
            ty,
            limit,
        )
    }
    /// Rebuild only a checked declared subobject at the recorded physical address.
    pub(crate) fn word_projection(
        &mut self,
        address: WordAddress,
        path: &[u64],
        expected: TypeId,
    ) -> Result<Constant, Failure> {
        let mut ty = Reader::new(self.facts()).location(address.root)?.ty;
        let mut offset = 0u64;
        for &index in path {
            let field = Reader::new(self.facts()).projection_layout(ty, index)?;
            offset = offset
                .checked_add(field.offset)
                .ok_or("symbolic typed projection offset overflow")?;
            ty = field.ty;
        }
        if offset != address.offset || !Reader::new(self.facts()).same_storage_type(ty, expected) {
            return Err("symbolic word does not select its checked typed subobject".into());
        }
        let mut pointer = super::super::references::Pointer::new(address.root.0);
        pointer.write = address.write;
        let mut value = Constant(Value::Reference(pointer));
        let mut ty = Reader::new(self.facts()).location(address.root)?.ty;
        for &index in path {
            ty = Reader::new(self.facts()).projection_layout(ty, index)?.ty;
            value = self.projection(value, index, ty)?;
        }
        Ok(value)
    }
}
impl Reader<'_> {
    pub(crate) fn sequence_displacement(
        self,
        parent: TypeId,
        from: u64,
        length: u64,
        selected: TypeId,
    ) -> Result<u64, &'static str> {
        let layout = self
            .facts
            .layouts
            .get(parent)
            .ok_or("missing prepared sequence layout")?;
        self.sequence_layout_displacement(parent, *layout, from, length, selected)
    }
    /// Lowering and execution may own distinct completed layout caches.
    pub(crate) fn sequence_layout_displacement(
        self,
        parent: TypeId,
        layout: crate::abi::layout::Layout,
        from: u64,
        length: u64,
        selected: TypeId,
    ) -> Result<u64, &'static str> {
        let Some(crate::types::Kind::Array {
            element,
            length: bound,
        }) = self.facts.types.kind(parent)
        else {
            return Err("symbolic range requires original array storage");
        };
        let crate::abi::layout::Shape::Array {
            element: actual,
            length: count,
            stride,
        } = layout.shape
        else {
            return Err("symbolic range requires retained array storage");
        };
        if actual != element || count != bound {
            return Err("symbolic range layout differs from its canonical array type");
        }
        if from.checked_add(length).is_none_or(|end| end > bound) {
            return Err("symbolic range exceeds its original allocation");
        }
        match self.facts.types.kind(selected) {
            Some(crate::types::Kind::Array {
                element: actual,
                length: count,
            }) if count == length && self.same_storage_type(element, actual) => {}
            Some(
                crate::types::Kind::Slice(_)
                | crate::types::Kind::Primitive(crate::types::Primitive::Str),
            ) if self.same_storage_type(element, self.sequence_element(selected)?) => {}
            _ => return Err("symbolic range has a different stored element type"),
        }
        from.checked_mul(stride)
            .ok_or("symbolic range byte offset overflow")
    }
}
