//! Typed views touch only their selected footprint; raw transport does not decode.
use super::{Backing, Constant, Failure, Id, Machine, Reader, TypeId};
use crate::sema::values::StoredShape;
use crate::{sema::members::VariantId, types::Kind};
struct Field {
    ty: TypeId,
    offset: u64,
}
enum Encode {
    Value {
        ty: TypeId,
        offset: u64,
        value: Constant,
    },
    Repeat {
        offset: u64,
        stride: u64,
        length: u64,
    },
}
enum Decode {
    Value {
        ty: TypeId,
        offset: u64,
    },
    Product {
        start: usize,
        variant: Option<VariantId>,
    },
    Repeat(u64),
}
impl Machine<'_, '_> {
    fn byte_fields(&mut self, ty: TypeId) -> Result<(Vec<Field>, Option<VariantId>), Failure> {
        self.footprint(ty)?;
        let (length, variant) = match Reader::new(self.evaluator.facts()).storage_shape(ty)? {
            StoredShape::Array { length, .. } => (length, None),
            StoredShape::Product {
                fields, variant, ..
            } => (fields.len() as u64, variant),
            _ => return Err("byte product requires aggregate storage".into()),
        };
        let length = usize::try_from(length).map_err(|_| "byte product exceeds host size")?;
        self.charge(length)?;
        self.reserve_cells(length.saturating_mul(4))?;
        let mut fields = Vec::with_capacity(length);
        for index in 0..length {
            let field = Reader::new(self.evaluator.facts()).projection_layout(ty, index as u64)?;
            fields.push(Field {
                ty: field.ty,
                offset: field.offset,
            });
        }
        Ok((fields, variant))
    }
    fn byte_union(&self, ty: TypeId) -> bool {
        matches!(
            Reader::new(self.evaluator.facts()).storage_shape(ty),
            Ok(StoredShape::Product { union: true, .. })
        )
    }
    pub(super) fn encode_bytes(
        &mut self,
        backing: &mut Backing,
        offset: u64,
        ty: TypeId,
        value: Constant,
    ) -> Result<(), Failure> {
        let mut pending = vec![Encode::Value { ty, offset, value }];
        while let Some(work) = pending.pop() {
            self.charge(1)?;
            match work {
                Encode::Value { ty, offset, value } => {
                    self.encode_byte_value(backing, offset, ty, value, &mut pending)?;
                }
                Encode::Repeat {
                    offset,
                    stride,
                    length,
                } => self.repeat_byte_span(backing, offset, stride, length)?,
            }
        }
        Ok(())
    }
    fn encode_byte_value(
        &mut self,
        backing: &mut Backing,
        offset: u64,
        ty: TypeId,
        value: Constant,
        pending: &mut Vec<Encode>,
    ) -> Result<(), Failure> {
        let layout = self.footprint(ty)?;
        let start = usize::try_from(offset).map_err(|_| "byte offset exceeds host size")?;
        let end = offset
            .checked_add(layout.size)
            .and_then(|n| usize::try_from(n).ok())
            .filter(|&n| n <= backing.bytes.len())
            .ok_or("byte encoding exceeds backing bounds")?;
        if start == end {
            return Ok(());
        }
        if matches!(
            self.evaluator.facts().types.kind(ty),
            Some(Kind::Primitive(_))
        ) {
            let (bits, width) = Reader::new(self.evaluator.facts()).encode_primitive(ty, value)?;
            if end - start != width {
                return Err("primitive bytes differ from retained layout".into());
            }
            self.charge(width)?;
            backing.bytes[start..end].copy_from_slice(&bits.to_le_bytes()[..width]);
            for index in start..end {
                backing.set_initialized(index, true);
            }
        } else if self.byte_union(ty) {
            let image = Reader::new(self.evaluator.facts()).image(value)?;
            if image.ty != ty || image.bytes.len() != end - start {
                return Err("union image differs from byte backing".into());
            }
            let width = image.bytes.len();
            self.charge(width)?;
            let image = Reader::new(self.evaluator.facts()).image(value)?;
            backing.bytes[start..end].copy_from_slice(image.bytes);
            for index in 0..width {
                backing.set_initialized(start + index, image.initialized(index));
            }
        } else if let StoredShape::Array {
            element,
            stride,
            length,
        } = Reader::new(self.evaluator.facts()).storage_shape(ty)?
            && let crate::sema::values::View::Repeat {
                value,
                length: actual,
            } = Reader::new(self.evaluator.facts()).view(value)?
        {
            if length != actual {
                return Err("repeated value differs from checked array length".into());
            }
            self.reserve_cells(pending.len().saturating_add(2).saturating_mul(4))?;
            pending.push(Encode::Repeat {
                offset,
                stride,
                length,
            });
            pending.push(Encode::Value {
                ty: element,
                offset,
                value,
            });
        } else {
            let (fields, _) = self.byte_fields(ty)?;
            self.reserve_cells(pending.len().saturating_add(fields.len()).saturating_mul(4))?;
            for (index, field) in fields.into_iter().enumerate().rev() {
                let value = self.evaluator.field(value, index as u64)?;
                pending.push(Encode::Value {
                    ty: field.ty,
                    offset: offset
                        .checked_add(field.offset)
                        .ok_or("byte field offset overflow")?,
                    value,
                });
            }
        }
        Ok(())
    }
    pub(super) fn repeat_byte_span(
        &mut self,
        backing: &mut Backing,
        offset: u64,
        stride: u64,
        length: u64,
    ) -> Result<(), Failure> {
        let total = stride
            .checked_mul(length)
            .ok_or("repeated byte extent overflow")?;
        let end = offset
            .checked_add(total)
            .and_then(|n| usize::try_from(n).ok())
            .filter(|&n| n <= backing.bytes.len())
            .ok_or("repeated bytes exceed backing bounds")?;
        if stride == 0 || length <= 1 {
            return Ok(());
        }
        let start =
            usize::try_from(offset).map_err(|_| "repeated byte offset exceeds host size")?;
        let stride =
            usize::try_from(stride).map_err(|_| "repeated byte stride exceeds host size")?;
        self.charge(end - start - stride)?;
        // This profile has no symbolic relocations. Enabling relocation families
        // must extend repetition to their transport metadata as well as bytes/masks.
        for target in (start + stride..end).step_by(stride) {
            backing.bytes.copy_within(start..start + stride, target);
            for index in 0..stride {
                let initialized = backing.initialized(start + index);
                backing.set_initialized(target + index, initialized);
            }
        }
        Ok(())
    }
    pub(super) fn encode_byte_cells(
        &mut self,
        backing: &mut Backing,
        cell: usize,
        ty: TypeId,
    ) -> Result<(), Failure> {
        let mut pending = vec![(cell, ty, 0u64)];
        while let Some((cell, ty, offset)) = pending.pop() {
            self.charge(1)?;
            match &self.memory.cells[cell] {
                super::Cell::Frozen(value) => {
                    let value = *value;
                    self.encode_bytes(backing, offset, ty, value)?;
                }
                super::Cell::Empty => {}
                super::Cell::Bytes(_) => {
                    return Err("nested byte backing escaped its allocation root".into());
                }
                super::Cell::Fields { values, .. } => {
                    let length = values.len();
                    self.reserve_cells(pending.len().saturating_add(length).saturating_mul(6))?;
                    let (fields, _) = self.byte_fields(ty)?;
                    let super::Cell::Fields { values, .. } = &self.memory.cells[cell] else {
                        return Err("byte conversion source changed".into());
                    };
                    if fields.len() != values.len() {
                        return Err("byte conversion fields differ from storage".into());
                    }
                    let children = values.clone();
                    for (cell, field) in children.into_iter().zip(fields).rev() {
                        pending.push((
                            cell,
                            field.ty,
                            offset
                                .checked_add(field.offset)
                                .ok_or("byte cell offset overflow")?,
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    pub(super) fn decode_bytes(
        &mut self,
        id: Id,
        offset: u64,
        ty: TypeId,
    ) -> Result<Constant, Failure> {
        let mut pending = vec![Decode::Value { ty, offset }];
        let mut values = Vec::new();
        while let Some(work) = pending.pop() {
            self.charge(1)?;
            match work {
                Decode::Product { start, variant } => {
                    let fields = values.split_off(start);
                    values.push(self.evaluator.fields(&fields, variant)?);
                }
                Decode::Repeat(length) => {
                    let value = values.pop().ok_or("missing repeated byte value")?;
                    values.push(self.evaluator.repeat(value, length)?);
                }
                Decode::Value { ty, offset } => {
                    let layout = self.footprint(ty)?;
                    let start =
                        usize::try_from(offset).map_err(|_| "byte read exceeds host size")?;
                    let end = offset
                        .checked_add(layout.size)
                        .and_then(|n| usize::try_from(n).ok())
                        .filter(|&n| n <= self.byte_backing(id).map_or(0, |b| b.bytes.len()))
                        .ok_or("byte read exceeds backing bounds")?;
                    if matches!(
                        self.evaluator.facts().types.kind(ty),
                        Some(Kind::Primitive(_))
                    ) {
                        values.push(self.decode_byte_scalar(id, start, end, ty)?);
                    } else if self.byte_union(ty) {
                        self.reserve_cells(
                            (end - start)
                                .saturating_mul(2)
                                .div_ceil(size_of::<Constant>())
                                .saturating_add(4),
                        )?;
                        let backing = self.byte_backing(id)?;
                        let mut mask = vec![0u64; (end - start).div_ceil(64)];
                        for index in 0..end - start {
                            if backing.initialized(start + index) {
                                mask[index / 64] |= 1u64 << (index % 64);
                            }
                        }
                        let bytes = backing.bytes[start..end].to_vec();
                        let (value, work) = self.evaluator.union_snapshot(
                            ty,
                            &bytes,
                            &mask,
                            self.limits.steps.saturating_sub(self.work.steps),
                        )?;
                        self.charge(work)?;
                        values.push(value);
                    } else if let Some(Kind::Array { element, length }) =
                        self.evaluator.facts().types.kind(ty)
                        && (length == 0 || self.footprint(element)?.size == 0)
                    {
                        if length == 0 {
                            values.push(self.evaluator.fields(&[], None)?);
                        } else {
                            pending.push(Decode::Repeat(length));
                            pending.push(Decode::Value {
                                ty: element,
                                offset,
                            });
                        }
                    } else {
                        let (fields, variant) = self.byte_fields(ty)?;
                        self.reserve_cells(
                            pending
                                .len()
                                .saturating_add(values.len())
                                .saturating_add(fields.len())
                                .saturating_mul(4),
                        )?;
                        pending.push(Decode::Product {
                            start: values.len(),
                            variant,
                        });
                        for field in fields.into_iter().rev() {
                            pending.push(Decode::Value {
                                ty: field.ty,
                                offset: offset
                                    .checked_add(field.offset)
                                    .ok_or("byte read field offset overflow")?,
                            });
                        }
                    }
                }
            }
        }
        if values.len() != 1 {
            return Err("byte decoding lacks one result".into());
        }
        values.pop().ok_or("missing decoded byte value".into())
    }
    fn decode_byte_scalar(
        &mut self,
        id: Id,
        start: usize,
        end: usize,
        ty: TypeId,
    ) -> Result<Constant, Failure> {
        let width = end - start;
        if width > 16 {
            return Err("primitive byte width exceeds scalar codec".into());
        }
        self.charge(width)?;
        let backing = self.byte_backing(id)?;
        let mut bits = [0; 16];
        let mut initialized = 0u64;
        bits[..width].copy_from_slice(&backing.bytes[start..end]);
        for index in 0..width {
            if backing.initialized(start + index) {
                initialized |= 1u64 << index;
            }
        }
        Ok(Reader::new(self.evaluator.facts()).decode_primitive(
            ty,
            &bits[..width],
            &[initialized],
        )?)
    }
}
