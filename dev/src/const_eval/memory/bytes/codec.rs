//! Typed views touch only their selected footprint; raw transport does not decode.
use super::continuations::{Duty, Key, Source};
use super::{Backing, Constant, Failure, Id, Machine, Read, Reader, TypeId};
use crate::const_eval::static_reads::ReadPurpose;
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
struct Decoder {
    pending: Vec<Decode>,
    values: Vec<Constant>,
    duties: Vec<Duty>,
    purpose: ReadPurpose,
    base_slots: usize,
    temporary: usize,
}
impl Decoder {
    fn retained_slots(&self) -> usize {
        self.pending
            .capacity()
            .saturating_mul(size_of::<Decode>())
            .saturating_add(self.values.capacity().saturating_mul(size_of::<Constant>()))
            .saturating_add(self.duties.capacity().saturating_mul(size_of::<Duty>()))
            .saturating_add(4 * size_of::<Self>())
            .div_ceil(size_of::<Constant>())
            .saturating_add(self.temporary)
    }
}
impl Machine<'_, '_> {
    fn byte_fields(
        &mut self,
        ty: TypeId,
        selected: Option<VariantId>,
    ) -> Result<(Vec<Field>, Option<VariantId>), Failure> {
        self.footprint(ty)?;
        if let Some(variant) = selected {
            let count = Reader::new(self.evaluator.facts())
                .enum_variant_fields(ty, variant)?
                .0
                .len();
            self.charge(count)?;
            self.reserve_cells(count.saturating_mul(4))?;
            let (fields, offsets) =
                Reader::new(self.evaluator.facts()).enum_variant_fields(ty, variant)?;
            let selected = fields
                .iter()
                .zip(offsets)
                .map(|(&ty, &offset)| Field { ty, offset })
                .collect();
            return Ok((selected, Some(variant)));
        }
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
        let ty = Reader::new(self.evaluator.facts()).storage_type(ty)?;
        let start = usize::try_from(offset).map_err(|_| "byte offset exceeds host size")?;
        let end = offset
            .checked_add(layout.size)
            .and_then(|n| usize::try_from(n).ok())
            .filter(|&n| n <= backing.bytes.len())
            .ok_or("byte encoding exceeds backing bounds")?;
        if start == end {
            self.check_product_value(ty, value)?;
            // Zero bytes do not erase nested callable capture identities. Walk
            // stored products and one Repeat recipe; empty arrays have no child.
            if !matches!(
                Reader::new(self.evaluator.facts()).storage_shape(ty)?,
                StoredShape::Product { union: false, .. } | StoredShape::Array { length: 1.., .. }
            ) {
                return Ok(());
            }
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
            self.buffer_fragments(backing, offset, width as u64, &[])?;
            backing.bytes[start..end].copy_from_slice(&bits.to_le_bytes()[..width]);
            for index in start..end {
                backing.set_initialized(index, true);
            }
        } else if matches!(
            Reader::new(self.evaluator.facts()).storage_shape(ty)?,
            StoredShape::Enumeration { .. } | StoredShape::Niche
        ) {
            self.encode_byte_enum(backing, offset, ty, value)?;
            let crate::sema::values::View::Product {
                variant: Some(variant),
                ..
            } = Reader::new(self.evaluator.facts()).view(value)?
            else {
                return Err("encoded enum lost its checked variant".into());
            };
            self.encode_byte_fields(ty, offset, value, Some(variant), pending)?;
        } else if matches!(
            Reader::new(self.evaluator.facts()).storage_shape(ty)?,
            StoredShape::Word
        ) {
            self.encode_byte_word(backing, offset, ty, value)?;
        } else if self.byte_union(ty)
            || (matches!(self.evaluator.facts().types.kind(ty), Some(Kind::Array { element, .. }) if element == TypeId::primitive(crate::types::Primitive::U8))
                && matches!(
                    Reader::new(self.evaluator.facts()).view(value)?,
                    crate::sema::values::View::Image { .. }
                ))
        {
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
            self.encode_image_fragments(backing, offset, value, width as u64)?;
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
            self.encode_byte_fields(ty, offset, value, None, pending)?;
        }
        Ok(())
    }
    fn encode_byte_fields(
        &mut self,
        ty: TypeId,
        offset: u64,
        value: Constant,
        variant: Option<VariantId>,
        pending: &mut Vec<Encode>,
    ) -> Result<(), Failure> {
        self.check_product_value(ty, value)?;
        let (fields, _) = self.byte_fields(ty, variant)?;
        self.reserve_cells(pending.len().saturating_add(fields.len()).saturating_mul(4))?;
        for (index, field) in fields.into_iter().enumerate().rev() {
            let value = self.stored_field(value, index as u64)?;
            pending.push(Encode::Value {
                ty: field.ty,
                offset: offset
                    .checked_add(field.offset)
                    .ok_or("byte field offset overflow")?,
                value,
            });
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
        let fragments = backing.fragments.as_ref().map_or_else(
            || Ok(Vec::new()),
            |tree| self.clip_fragments(tree, offset, stride as u64),
        )?;
        for target in (start + stride..end).step_by(stride) {
            self.buffer_fragments(backing, target as u64, stride as u64, &fragments)?;
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
            let ty = Reader::new(self.evaluator.facts()).storage_type(ty)?;
            match &self.memory.cells[cell] {
                super::Cell::Frozen(value) => {
                    let value = *value;
                    self.encode_bytes(backing, offset, ty, value)?;
                }
                super::Cell::Empty => {}
                super::Cell::Bytes(_) => {
                    return Err("nested byte backing escaped its allocation root".into());
                }
                super::Cell::Fields { values, variant } => {
                    let length = values.len();
                    let variant = *variant;
                    self.reserve_cells(pending.len().saturating_add(length).saturating_mul(6))?;
                    let enumeration = matches!(
                        Reader::new(self.evaluator.facts()).storage_shape(ty)?,
                        StoredShape::Enumeration { .. } | StoredShape::Niche
                    );
                    let selected = if enumeration {
                        let id = variant.ok_or("stored enum cells lack their exact variant")?;
                        self.encode_byte_enum_tag(backing, offset, ty, id)?;
                        Some(id)
                    } else {
                        None
                    };
                    let (fields, expected) = self.byte_fields(ty, selected)?;
                    if matches!(self.evaluator.facts().types.kind(ty), Some(Kind::Adt(owner,_)) if self.evaluator.facts().aggregates.get(owner).is_some_and(|definition| definition.kind == crate::types::aggregate::Kind::Enum))
                        && variant != expected
                    {
                        return Err("stored enum cells differ from their canonical variant".into());
                    }
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
        purpose: ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        self.decode_source(Source::Backing(id), offset, ty, purpose)
    }
    pub(super) fn decode_source(
        &mut self,
        source: Source,
        offset: u64,
        ty: TypeId,
        purpose: ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        let key = Key {
            source,
            offset,
            ty,
            length: None,
            purpose,
            context: self.context,
        };
        if let Some(value) = self.resumed_byte_read(key)? {
            return Ok(value);
        }
        let (value, duties) = self.decode_byte_storage(source, offset, ty, purpose)?;
        self.complete_byte_read(key, value, duties)
    }
    fn decode_byte_storage(
        &mut self,
        source: Source,
        offset: u64,
        ty: TypeId,
        purpose: ReadPurpose,
    ) -> Result<(Constant, Vec<Duty>), Failure> {
        self.reserve_cells(16)?;
        self.decode_byte_work(source, vec![Decode::Value { ty, offset }], purpose)
    }
    pub(super) fn decode_byte_range(
        &mut self,
        id: Id,
        offset: u64,
        ty: TypeId,
        range: crate::const_eval::memory::Range,
        purpose: ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        self.decode_source_range(Source::Backing(id), offset, ty, range, purpose)
    }
    pub(super) fn decode_source_range(
        &mut self,
        source: Source,
        offset: u64,
        ty: TypeId,
        range: crate::const_eval::memory::Range,
        purpose: ReadPurpose,
    ) -> Result<Read<Constant>, Failure> {
        let StoredShape::Array {
            element,
            stride,
            length,
        } = Reader::new(self.evaluator.facts()).storage_shape(ty)?
        else {
            return Err("byte range requires retained array storage".into());
        };
        if range
            .from
            .checked_add(range.length)
            .is_none_or(|end| end > length)
        {
            return Err("byte range exceeds backing bounds".into());
        }
        let offset = offset
            .checked_add(
                stride
                    .checked_mul(range.from)
                    .ok_or("byte range displacement overflow")?,
            )
            .ok_or("byte range offset overflow")?;
        let key = Key {
            source,
            offset,
            ty,
            length: Some(range.length),
            purpose,
            context: self.context,
        };
        if let Some(value) = self.resumed_byte_read(key)? {
            return Ok(value);
        }
        if purpose == ReadPurpose::Value
            && element == TypeId::primitive(crate::types::Primitive::U8)
            && self.source_has_fragments(source, offset, range.length)?
        {
            self.charge(1)?;
            let selected = self
                .evaluator
                .byte_array_range_type(ty, range.from, range.length)?;
            let start = usize::try_from(offset).map_err(|_| "byte range exceeds host size")?;
            let end = offset
                .checked_add(range.length)
                .and_then(|n| usize::try_from(n).ok())
                .filter(|&end| end <= self.byte_data(source).map_or(0, |data| data.bytes.len()))
                .ok_or("byte range extent overflow")?;
            // The selected array type and full source bounds remain independently checked.
            let value = self.decode_byte_image(source, start, end, selected)?;
            return Ok(Read::Value(value));
        }
        let count = if stride == 0 && range.length != 0 {
            1
        } else {
            range.length
        };
        let count = usize::try_from(count).map_err(|_| "byte range exceeds host size")?;
        self.reserve_cells(
            count
                .saturating_add(1)
                .saturating_mul(size_of::<Decode>())
                .div_ceil(size_of::<Constant>())
                .saturating_add(16),
        )?;
        let mut pending = Vec::new();
        pending
            .try_reserve_exact(count.saturating_add(1))
            .map_err(|_| "byte range decode work allocation failed")?;
        pending.push(if stride == 0 && range.length != 0 {
            Decode::Repeat(range.length)
        } else {
            Decode::Product {
                start: 0,
                variant: None,
            }
        });
        self.charge(count)?;
        for index in (0..count).rev() {
            pending.push(Decode::Value {
                ty: element,
                offset: offset
                    .checked_add(
                        stride
                            .checked_mul(index as u64)
                            .ok_or("byte range extent overflow")?,
                    )
                    .ok_or("byte range offset overflow")?,
            });
        }
        let (value, duties) = self.decode_byte_work(source, pending, purpose)?;
        self.complete_byte_read(key, value, duties)
    }
    fn decode_byte_work(
        &mut self,
        source: Source,
        pending: Vec<Decode>,
        purpose: ReadPurpose,
    ) -> Result<(Constant, Vec<Duty>), Failure> {
        let mut state = Decoder {
            pending,
            values: Vec::new(),
            duties: Vec::new(),
            purpose,
            base_slots: self.slots,
            temporary: 0,
        };
        let previous = self.slots;
        let result = (|| {
            while let Some(work) = state.pending.pop() {
                self.slots = previous
                    .checked_add(state.retained_slots())
                    .ok_or("byte decoder temporary retention overflow")?;
                self.reserve_cells(0)?;
                self.charge(1)?;
                match work {
                    Decode::Product { start, variant } => {
                        self.reserve_cells(state.values.len().saturating_sub(start))?;
                        let fields = state.values.split_off(start);
                        state.temporary = fields.capacity();
                        self.reserve_decode(&mut state, 1, 0, 0)?;
                        state.values.push(self.evaluator.fields(&fields, variant)?);
                        state.temporary = 0;
                    }
                    Decode::Repeat(length) => {
                        let value = state.values.pop().ok_or("missing repeated byte value")?;
                        self.reserve_decode(&mut state, 1, 0, 0)?;
                        state.values.push(self.evaluator.repeat(value, length)?);
                    }
                    Decode::Value { ty, offset } => {
                        self.decode_byte_value(source, offset, ty, &mut state)?;
                    }
                }
            }
            if state.values.len() != 1 {
                return Err("byte decoding lacks one result".into());
            }
            state
                .values
                .pop()
                .ok_or("missing decoded byte value".into())
        })();
        self.slots = previous;
        result.map(|value| (value, state.duties))
    }
    fn reserve_decode(
        &mut self,
        state: &mut Decoder,
        values: usize,
        pending: usize,
        duties: usize,
    ) -> Result<(), Failure> {
        fn growth<T>(values: &Vec<T>, additional: usize) -> usize {
            if values.capacity().saturating_sub(values.len()) >= additional {
                0
            } else {
                values
                    .len()
                    .saturating_add(additional)
                    .max(values.capacity().saturating_mul(2))
                    .max(4)
                    .saturating_mul(size_of::<T>())
            }
        }
        self.slots = state
            .base_slots
            .checked_add(state.retained_slots())
            .ok_or("byte decoder retention overflow")?;
        let bytes = growth(&state.values, values)
            .saturating_add(growth(&state.pending, pending))
            .saturating_add(growth(&state.duties, duties));
        self.reserve_cells(bytes.div_ceil(size_of::<Constant>()))?;
        state
            .values
            .try_reserve(values)
            .map_err(|_| "byte decoded values allocation failed")?;
        state
            .pending
            .try_reserve(pending)
            .map_err(|_| "byte decode work allocation failed")?;
        state
            .duties
            .try_reserve(duties)
            .map_err(|_| "byte reference duties allocation failed")?;
        self.slots = state
            .base_slots
            .checked_add(state.retained_slots())
            .ok_or("byte decoder retention overflow")?;
        self.reserve_cells(0)
    }
    fn decode_byte_value(
        &mut self,
        source: Source,
        offset: u64,
        ty: TypeId,
        state: &mut Decoder,
    ) -> Result<(), Failure> {
        #[cfg(test)]
        {
            self.memory.bytes.decoded += 1;
        }
        let layout = self.footprint(ty)?;
        let ty = Reader::new(self.evaluator.facts()).storage_type(ty)?;
        let start = usize::try_from(offset).map_err(|_| "byte read exceeds host size")?;
        let end = offset
            .checked_add(layout.size)
            .and_then(|n| usize::try_from(n).ok())
            .filter(|&n| n <= self.byte_data(source).map_or(0, |b| b.bytes.len()))
            .ok_or("byte read exceeds backing bounds")?;
        self.reserve_decode(state, 1, 0, 0)?;
        if matches!(
            self.evaluator.facts().types.kind(ty),
            Some(Kind::Primitive(_))
        ) {
            state
                .values
                .push(self.decode_byte_scalar(source, start, end, ty)?);
        } else if matches!(
            Reader::new(self.evaluator.facts()).storage_shape(ty)?,
            StoredShape::Enumeration { .. } | StoredShape::Niche
        ) {
            let variant = self.decode_byte_variant(source, offset, ty)?;
            self.decode_byte_fields(ty, offset, Some(variant), state)?;
        } else if matches!(
            Reader::new(self.evaluator.facts()).storage_shape(ty)?,
            StoredShape::Word
        ) {
            let (value, duty) = self.decode_byte_word(source, offset, ty)?;
            if let Some(duty) = duty
                && state.purpose == ReadPurpose::Value
            {
                self.reserve_decode(state, 0, 0, 1)?;
                state.duties.push(duty);
            }
            state.values.push(value);
        } else if self.byte_union(ty)
            || (state.purpose == ReadPurpose::Value
                && matches!(self.evaluator.facts().types.kind(ty), Some(Kind::Array { element, .. }) if element == TypeId::primitive(crate::types::Primitive::U8))
                && self.source_has_fragments(source, offset, layout.size)?)
        {
            state
                .values
                .push(self.decode_byte_image(source, start, end, ty)?);
        } else if let Some(Kind::Array { element, length }) = self.evaluator.facts().types.kind(ty)
            && (length == 0 || self.footprint(element)?.size == 0)
        {
            if length == 0 {
                state.values.push(self.evaluator.fields(&[], None)?);
            } else {
                self.reserve_decode(state, 0, 2, 0)?;
                state.pending.push(Decode::Repeat(length));
                state.pending.push(Decode::Value {
                    ty: element,
                    offset,
                });
            }
        } else {
            self.decode_byte_fields(ty, offset, None, state)?;
        }
        Ok(())
    }
    fn decode_byte_fields(
        &mut self,
        ty: TypeId,
        offset: u64,
        selected: Option<VariantId>,
        state: &mut Decoder,
    ) -> Result<(), Failure> {
        let (fields, variant) = self.byte_fields(ty, selected)?;
        state.temporary = fields
            .capacity()
            .saturating_mul(size_of::<Field>())
            .div_ceil(size_of::<Constant>());
        self.reserve_decode(state, 0, fields.len().saturating_add(1), 0)?;
        state.pending.push(Decode::Product {
            start: state.values.len(),
            variant,
        });
        for field in fields.into_iter().rev() {
            state.pending.push(Decode::Value {
                ty: field.ty,
                offset: offset
                    .checked_add(field.offset)
                    .ok_or("byte read field offset overflow")?,
            });
        }
        state.temporary = 0;
        Ok(())
    }
    fn decode_byte_image(
        &mut self,
        source: Source,
        start: usize,
        end: usize,
        ty: TypeId,
    ) -> Result<Constant, Failure> {
        self.reserve_cells(
            (end - start)
                .saturating_mul(2)
                .div_ceil(size_of::<Constant>())
                .saturating_add(4),
        )?;
        let backing = self.byte_data(source)?;
        let mut mask = vec![0u64; (end - start).div_ceil(64)];
        for index in 0..end - start {
            if backing.initialized(start + index) {
                mask[index / 64] |= 1u64 << (index % 64);
            }
        }
        let bytes = backing.bytes[start..end].to_vec();
        let previous = self.slots;
        self.slots = previous
            .checked_add(
                (bytes.capacity() + mask.capacity() * size_of::<u64>())
                    .div_ceil(size_of::<Constant>()),
            )
            .ok_or("byte image temporary retention overflow")?;
        let result = (|| {
            let fragments = self.source_word_targets(source, start as u64, (end - start) as u64)?;
            self.slots = self
                .slots
                .checked_add(
                    fragments
                        .capacity()
                        .saturating_mul(size_of::<crate::sema::values::ImageFragment>())
                        .div_ceil(size_of::<Constant>()),
                )
                .ok_or("image word snapshot retention overflow")?;
            let limit = self.limits.steps.saturating_sub(self.work.steps);
            let (value, work) = if self.byte_union(ty) {
                self.evaluator
                    .union_snapshot(ty, &bytes, &mask, &fragments, limit)?
            } else {
                self.evaluator
                    .byte_array_snapshot(ty, &bytes, &mask, &fragments, limit)?
            };
            self.charge(work)?;
            Ok(value)
        })();
        self.slots = previous;
        result
    }
    pub(super) fn decode_byte_scalar(
        &mut self,
        source: Source,
        start: usize,
        end: usize,
        ty: TypeId,
    ) -> Result<Constant, Failure> {
        let width = end
            .checked_sub(start)
            .ok_or("primitive byte span is reversed")?;
        if width > 16 {
            return Err("primitive byte width exceeds scalar codec".into());
        }
        self.charge(width)?;
        if self.source_has_fragments(source, start as u64, width as u64)? {
            return Err("constant numeric read uses symbolic pointer bytes".into());
        }
        let backing = self.byte_data(source)?;
        let mut bits = [0; 16];
        let mut initialized = 0u64;
        bits[..width].copy_from_slice(
            backing
                .bytes
                .get(start..end)
                .ok_or("primitive byte read exceeds storage")?,
        );
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
