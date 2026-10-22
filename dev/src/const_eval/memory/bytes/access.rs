//! All views of a converted root resolve against its sole mutable backing.
use super::{Constant, Failure, Id, Machine, Read, Reader, TypeId};
use crate::{
    const_eval::memory::{Range, Resolved},
    sema::members::Selection,
};
impl Machine<'_, '_> {
    pub(in crate::const_eval::memory) fn freeze_byte_root(
        &mut self,
        id: Id,
        purpose: crate::const_eval::static_reads::ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        let ty = self.byte_backing(id)?.ty;
        self.decode_bytes(id, 0, ty, purpose)
    }
    pub(in crate::const_eval::memory) fn check_byte_length(
        &mut self,
        root: crate::sema::values::LocationId,
        path: &[u64],
    ) -> Result<(), Failure> {
        let ty = Reader::new(self.evaluator.facts()).location(root)?.ty;
        self.storage_offset(ty, path)?;
        Ok(())
    }
    fn byte_selection(
        &mut self,
        place: Resolved,
    ) -> Result<(crate::sema::values::LocationId, TypeId, u64), Failure> {
        let (root, path) = self.path(place.address)?.fields_only()?;
        let root_ty = Reader::new(self.evaluator.facts()).location(root)?.ty;
        let (mut ty, mut offset) = self.storage_offset(root_ty, &path)?;
        if let Some(field) = place.union {
            if ty != field.owner {
                return Err("byte union field has a different owner".into());
            }
            let field = self.checked_byte_union(field)?;
            ty = field.0;
            offset = offset
                .checked_add(field.1)
                .ok_or("byte union offset overflow")?;
        }
        Ok((root, ty, offset))
    }
    fn checked_byte_union(
        &self,
        union: crate::const_eval::memory::UnionField,
    ) -> Result<(TypeId, u64), Failure> {
        let Selection::Nominal(field) = union.field else {
            return Err("byte union field lacks nominal identity".into());
        };
        let ordinal = self
            .evaluator
            .facts()
            .members
            .ordinal(field)
            .ok_or("missing byte union ordinal")?;
        let selected = Reader::new(self.evaluator.facts())
            .projection_layout(union.owner, u64::from(ordinal))?;
        if !matches!(selected.storage,crate::sema::values::ProjectionStorage::UnionField{field} if field==union.field)
        {
            return Err("byte union field differs from its canonical owner".into());
        }
        Ok((selected.ty, selected.offset))
    }
    pub(in crate::const_eval::memory) fn read_byte_union(
        &mut self,
        root: crate::sema::values::LocationId,
        path: &[u64],
        union: crate::const_eval::memory::UnionField,
        leaf: TypeId,
        purpose: crate::const_eval::static_reads::ReadPurpose,
    ) -> Result<Option<Read<Constant>>, Failure> {
        let Some(id) = self.byte_root(root) else {
            return Ok(None);
        };
        let root_ty = self.byte_backing(id)?.ty;
        let (owner, offset) = self.storage_offset(root_ty, path)?;
        if owner != union.owner {
            return Err("byte union read has a different owner".into());
        }
        let (ty, delta) = self.checked_byte_union(union)?;
        if !self.image_field_compatible(ty, leaf) {
            return Err("byte union read differs from its selected field type".into());
        }
        self.decode_bytes(
            id,
            offset
                .checked_add(delta)
                .ok_or("byte union read offset overflow")?,
            ty,
            purpose,
        )
        .map(Some)
    }
    pub(in crate::const_eval::memory) fn write_byte_place(
        &mut self,
        place: Resolved,
        value: Constant,
    ) -> Result<bool, Failure> {
        if place.union.is_none() {
            return Ok(false);
        }
        let root = self.path(place.address)?.root;
        if self.byte_root(root).is_none() {
            let union = place.union.ok_or("missing selected union storage")?;
            if Reader::new(self.evaluator.facts()).scalar_union(union.owner)? {
                return Ok(false);
            }
            self.convert_byte_root(root)?;
        }
        let (root, ty, offset) = self.byte_selection(place)?;
        self.write_byte_value(root, offset, ty, value)?;
        Ok(true)
    }
    pub(in crate::const_eval::memory) fn write_byte_path(
        &mut self,
        path: &crate::const_eval::memory::StoragePath,
        value: Constant,
    ) -> Result<bool, Failure> {
        if self.byte_root(path.root).is_none() {
            if !Reader::new(self.evaluator.facts()).has_image_families() || path.fields.is_empty() {
                return Ok(false);
            }
            if !self.writes_inside_image(path.root, &path.fields)? {
                return Ok(false);
            }
            self.convert_byte_root(path.root)?;
        }
        if let Some(range) = path.range {
            return self.write_byte_range(path.root, &path.fields, range, value);
        }
        let ty = Reader::new(self.evaluator.facts()).location(path.root)?.ty;
        let (ty, offset) = self.storage_offset(ty, &path.fields)?;
        self.write_byte_value(path.root, offset, ty, value)?;
        Ok(true)
    }
    fn writes_inside_image(
        &mut self,
        root: crate::sema::values::LocationId,
        path: &[u64],
    ) -> Result<bool, Failure> {
        let Some(&root) = self.memory.roots.get(&root) else {
            return Ok(false);
        };
        let mut cell = root;
        let mut frozen = None;
        for &index in path {
            self.charge(1)?;
            let value = if let Some(value) = frozen {
                value
            } else {
                match &self.memory.cells[cell] {
                    super::super::Cell::Frozen(value) => *value,
                    super::super::Cell::Fields { values, .. } => {
                        cell = usize::try_from(index)
                            .ok()
                            .and_then(|index| values.get(index))
                            .copied()
                            .ok_or("constant index is out of bounds")?;
                        continue;
                    }
                    super::super::Cell::Bytes(_) => {
                        return Err("byte root bypassed its write consumer".into());
                    }
                    super::super::Cell::Empty => return Ok(false),
                }
            };
            if matches!(
                Reader::new(self.evaluator.facts()).view(value)?,
                crate::sema::values::View::Image { .. }
            ) {
                return Ok(true);
            }
            frozen = Some(self.evaluator.field(value, index)?);
        }
        Ok(false)
    }
    fn write_byte_value(
        &mut self,
        root: crate::sema::values::LocationId,
        offset: u64,
        ty: TypeId,
        value: Constant,
    ) -> Result<(), Failure> {
        let mut buffer = self.empty_backing(ty)?;
        let slots = buffer.retained_slots();
        let previous = self.slots;
        self.slots = previous
            .checked_add(slots)
            .ok_or("byte temporary retention overflow")?;
        let result = self
            .encode_bytes(&mut buffer, 0, ty, value)
            .and_then(|()| self.commit_byte_snapshot(root, offset, &buffer.snapshot()));
        self.slots = previous;
        result
    }
    pub(in crate::const_eval::memory) fn read_byte_range(
        &mut self,
        root: crate::sema::values::LocationId,
        path: &[u64],
        range: Range,
        purpose: crate::const_eval::static_reads::ReadPurpose,
    ) -> Result<Option<Read<Constant>>, Failure> {
        let Some(id) = self.byte_root(root) else {
            return Ok(None);
        };
        let ty = self.byte_backing(id)?.ty;
        let (ty, offset) = self.storage_offset(ty, path)?;
        self.decode_byte_range(id, offset, ty, range, purpose)
            .map(Some)
    }
    pub(in crate::const_eval::memory) fn write_byte_range(
        &mut self,
        root: crate::sema::values::LocationId,
        path: &[u64],
        range: Range,
        value: Constant,
    ) -> Result<bool, Failure> {
        let Some(id) = self.byte_root(root) else {
            return Ok(false);
        };
        let root_ty = self.byte_backing(id)?.ty;
        let (ty, offset) = self.storage_offset(root_ty, path)?;
        let actual = Reader::new(self.evaluator.facts()).field_types(ty, value)?;
        let length = match actual {
            crate::sema::values::FieldTypes::Repeated { length, .. } => length,
            crate::sema::values::FieldTypes::Sequence(fields) => fields.len() as u64,
        };
        // The assignment value has the selected range length, not the original array length.
        let value_length = match Reader::new(self.evaluator.facts()).view(value)? {
            crate::sema::values::View::Product { length, .. }
            | crate::sema::values::View::Repeat { length, .. }
            | crate::sema::values::View::ByteRange { length, .. } => length,
            crate::sema::values::View::Bytes(text) => {
                Reader::new(self.evaluator.facts()).bytes(text)?.len() as u64
            }
            _ => return Err("byte range assignment requires sequence value".into()),
        };
        if value_length != range.length
            || range
                .from
                .checked_add(range.length)
                .is_none_or(|end| end > length)
        {
            return Err("byte range assignment exceeds its storage".into());
        }
        let crate::sema::values::StoredShape::Array {
            element, stride, ..
        } = Reader::new(self.evaluator.facts()).storage_shape(ty)?
        else {
            return Err("byte range assignment requires array backing".into());
        };
        let size = range
            .length
            .checked_mul(stride)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("byte range assignment size overflow")?;
        let mut buffer = self.empty_byte_buffer(ty, size)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(buffer.retained_slots())
            .ok_or("byte range retention overflow")?;
        let result = (|| {
            if let crate::sema::values::View::Repeat {
                value: fill,
                length,
            } = Reader::new(self.evaluator.facts()).view(value)?
            {
                if length != range.length {
                    return Err("byte range repetition differs from assignment".into());
                }
                if stride != 0 && length != 0 {
                    self.encode_bytes(&mut buffer, 0, element, fill)?;
                }
                self.repeat_byte_span(&mut buffer, 0, stride, length)?;
            } else if stride != 0 {
                for index in 0..range.length {
                    self.charge(1)?;
                    let field = self.evaluator.field(value, index)?;
                    self.encode_bytes(
                        &mut buffer,
                        index
                            .checked_mul(stride)
                            .ok_or("byte range offset overflow")?,
                        element,
                        field,
                    )?;
                }
            }
            Ok::<_, Failure>(())
        })();
        let result = result.and_then(|()| {
            let start = range
                .from
                .checked_mul(stride)
                .and_then(|n| offset.checked_add(n))
                .ok_or("byte range destination offset overflow")?;
            self.commit_byte_snapshot(root, start, &buffer.snapshot())?;
            Ok(true)
        });
        self.slots = previous;
        result
    }
}
