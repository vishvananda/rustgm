//! Serialize completed constants in canonical byte order, with bounded flat work.
mod images;
use super::{Context, Fixup, Key, Target, Type};
use crate::{
    abi::layout::{Layout, Shape},
    literal::value::FloatBits,
    lowir::data::{GlobalStorage, Item},
    sema::{
        model::Scalar,
        values::{Constant, Metadata, View},
    },
    types::{Kind, TypeId},
};

#[derive(Clone, Copy)]
enum Work {
    Value {
        offset: u64,
        ty: TypeId,
        value: Constant,
    },
    Bits {
        offset: u64,
        bytes: u64,
        bits: u128,
    },
}
struct Initializer {
    global: crate::lowir::data::GlobalId,
    items: Vec<Item>,
    cursor: u64,
    align: u32,
    work: Vec<Work>,
}
impl Context<'_> {
    pub fn initialize_data(&mut self, index: usize) -> Result<(), &'static str> {
        let object = self.data.pending[index];
        let span = self.output.globals[object.global.0 as usize].span;
        let mut init = Initializer {
            global: object.global,
            items: Vec::new(),
            cursor: 0,
            align: span.align,
            work: vec![Work::Value {
                offset: 0,
                ty: object.ty,
                value: object.value,
            }],
        };
        while let Some(work) = init.work.pop() {
            self.data.charge(1)?;
            match work {
                Work::Bits {
                    offset,
                    bytes,
                    bits,
                } => init.bits(offset, bytes, bits)?,
                Work::Value { offset, ty, value } => {
                    self.initialize_value(&mut init, offset, ty, value)?;
                }
            }
        }
        init.pad(span.bytes)?;
        self.output.initialize(object.global, &init.items)
    }
    fn initialize_value(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        ty: TypeId,
        value: Constant,
    ) -> Result<(), &'static str> {
        let layout = self.layout(ty)?;
        if layout.size == 0 {
            return Ok(());
        }
        match self.constants.view(value)? {
            View::Scalar(scalar) => init.bits(offset, layout.size, bits(scalar)),
            View::Image { size } => self.initialize_image(init, offset, ty, value, size),
            View::Address { bits } => {
                init.bits(offset, 8, u128::from(bits))?;
                if matches!(layout.shape, Shape::WidePointer) {
                    self.initialize_metadata(
                        init,
                        offset + 8,
                        value
                            .metadata()
                            .ok_or("wide absolute pointer lacks metadata")?,
                    )?;
                }
                Ok(())
            }
            View::BytePointer { text } => {
                self.initialize_text(init, offset, text, false)?;
                self.initialize_metadata(
                    init,
                    offset + 8,
                    value.metadata().ok_or("wide byte pointer lacks metadata")?,
                )
            }
            View::Function(id) => {
                let symbol = self.demand_reified(id)?;
                self.initialize_pointer(init, offset, Target::Function(symbol), 0)
            }
            View::Reference(id) => self.initialize_reference(
                init,
                offset,
                id,
                matches!(layout.shape, Shape::WidePointer),
                value.metadata(),
            ),
            View::Bytes(id) => {
                if let Some(Kind::Reference { .. } | Kind::Pointer { .. }) = self.types.kind(ty) {
                    return self.initialize_text(
                        init,
                        offset,
                        id,
                        matches!(layout.shape, Shape::WidePointer),
                    );
                }
                self.initialize_bytes(init, offset, ty, value)
            }
            View::ByteRange { .. } => self.initialize_bytes(init, offset, ty, value),
            View::Text(id) => {
                self.initialize_text(init, offset, id, matches!(layout.shape, Shape::WidePointer))
            }
            View::Repeat {
                value: repeated,
                length,
            } => self.initialize_repeat(init, offset, layout, repeated, length),
            View::Product { length, .. } => {
                self.initialize_product(init, offset, &layout, value, length)
            }
        }
    }
    /// A tuple, struct, enum variant or array: its fields, then any niche or tag.
    fn initialize_product(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        layout: &Layout,
        value: Constant,
        length: u64,
    ) -> Result<(), &'static str> {
        self.data
            .charge(usize::try_from(length).map_err(|_| "constant product exceeds host size")?)?;
        if let Some(bytes) = self.byte_elements(init, layout, value, length)? {
            return init.byte_run(offset, bytes.len(), |index| bytes[index]);
        }
        for index in (0..length).rev() {
            let (delta, ty) = self.constant_field(layout, Some(value), index)?;
            init.work.push(Work::Value {
                offset: offset + delta,
                ty,
                value: self.constants.field(value, index)?,
            });
        }
        if let Some(niche) = self.layouts.niche(layout)? {
            let ordinal = self.constant_variant(value)?;
            // A zero-sized companion's one field writes no bytes.
            if ordinal == niche.empty as usize && length <= 1 {
                init.work.push(Work::Bits {
                    offset: offset + niche.offset,
                    bytes: niche.carrier.bytes(),
                    bits: niche.carrier.empty_bits(self.types)?,
                });
            } else if ordinal != niche.payload as usize || length != 1 {
                return Err("nullable constant differs from its canonical variant");
            }
        }
        if let Shape::Aggregate { tag: Some(tag), .. } = layout.shape {
            let variant = self
                .layouts
                .variants(layout)
                .get(self.constant_variant(value)?)
                .ok_or("constant enum lacks its variant layout")?;
            let bits = variant.discriminant;
            let bytes = self.layout(TypeId::primitive(tag.ty))?.size;
            init.work.push(Work::Bits {
                offset: offset + tag.offset,
                bytes,
                bits,
            });
        }
        Ok(())
    }
    /// The elements of a byte array long enough for word items, when each is
    /// a scalar; otherwise `None`, and the array is written element by element.
    fn byte_elements(
        &mut self,
        init: &Initializer,
        layout: &Layout,
        value: Constant,
        length: u64,
    ) -> Result<Option<Vec<u8>>, &'static str> {
        if init.align < 8
            || length < super::WORD_RUN
            || !matches!(layout.shape, Shape::Array { stride: 1, .. })
        {
            return Ok(None);
        }
        let mut bytes = Vec::new();
        for index in 0..length {
            let View::Scalar(scalar) = self.constants.view(self.constants.field(value, index)?)?
            else {
                return Ok(None);
            };
            bytes.push(u8::try_from(bits(scalar)).map_err(|_| "constant byte exceeds its width")?);
        }
        Ok(Some(bytes))
    }
    fn initialize_bytes(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        ty: TypeId,
        value: Constant,
    ) -> Result<(), &'static str> {
        let bytes = self.constants.stored_bytes(ty, value)?;
        self.data.charge(bytes.len())?;
        init.byte_run(offset, bytes.len(), |index| bytes[index])
    }
    fn initialize_repeat(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        layout: Layout,
        repeated: Constant,
        length: u64,
    ) -> Result<(), &'static str> {
        let Shape::Array {
            element,
            stride,
            length: actual,
        } = layout.shape
        else {
            return Err("constant repetition requires an array layout");
        };
        if actual != length {
            return Err("constant repeat length differs from its type");
        }
        if self.zero_data(element, repeated)? {
            init.pad(offset)?;
            init.zero(layout.size)?;
            return Ok(());
        }
        let count = usize::try_from(length).map_err(|_| "constant repeat exceeds host size")?;
        self.data.charge(count)?;
        if stride == 1
            && let View::Scalar(scalar) = self.constants.view(repeated)?
        {
            let byte = u8::try_from(bits(scalar)).map_err(|_| "constant byte exceeds its width")?;
            return init.byte_run(offset, count, |_| byte);
        }
        for index in (0..length).rev() {
            init.work.push(Work::Value {
                offset: offset + index * stride,
                ty: element,
                value: repeated,
            });
        }
        Ok(())
    }
    fn initialize_reference(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        id: crate::sema::values::LocationId,
        wide: bool,
        metadata: Option<Metadata>,
    ) -> Result<(), &'static str> {
        let pointer = self.constant_pointer(id)?;
        let addend =
            i64::try_from(pointer.offset).map_err(|_| "constant address offset overflow")?;
        self.initialize_pointer(init, offset, Target::Global(pointer.global), addend)?;
        if wide {
            if let Some(metadata) = metadata {
                return self.initialize_metadata(init, offset + 8, metadata);
            }
            let location = self.constants.location(id)?;
            let length = if let Some(Kind::Array { length, .. }) = self.types.kind(location.ty) {
                length
            } else {
                let value = location
                    .value
                    .ok_or("wide reference lacks backing metadata")?;
                let (View::Text(text) | View::Bytes(text)) = self.constants.view(value)? else {
                    return Err("wide reference lacks array or text metadata");
                };
                self.constants.bytes(text)?.len() as u64
            };
            init.bits(offset + 8, 8, u128::from(length))?;
        }
        Ok(())
    }
    fn initialize_metadata(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        metadata: Metadata,
    ) -> Result<(), &'static str> {
        match metadata {
            Metadata::Length(length) => init.bits(offset, 8, u128::from(length)),
            Metadata::Object { concrete, target } => {
                let table = self.object_table(concrete, target)?;
                self.initialize_pointer(init, offset, Target::Global(table), 0)
            }
        }
    }
    pub fn bytes_data(
        &mut self,
        id: crate::sema::text::Id,
    ) -> Result<crate::lowir::data::GlobalId, &'static str> {
        let key = Key::Bytes(id);
        if let Some(&global) = self.data.globals.get(&key) {
            return Ok(global);
        }
        let bytes = self.constants.bytes(id)?;
        self.data.charge(bytes.len().saturating_add(1))?;
        let span = super::definition_span(bytes.len() as u64, 1, false);
        let global = self.output.global(
            self.internal_name(format!("rg.data.{}", self.output.globals.len())),
            span,
            GlobalStorage::Readonly,
        )?;
        let mut init = Initializer {
            global,
            items: Vec::new(),
            cursor: 0,
            align: span.align,
            work: Vec::new(),
        };
        init.byte_run(0, bytes.len(), |index| bytes[index])?;
        init.pad(span.bytes)?;
        self.output.initialize(global, &init.items)?;
        self.data.globals.insert(key, global);
        Ok(global)
    }
    fn initialize_text(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        id: crate::sema::text::Id,
        wide: bool,
    ) -> Result<(), &'static str> {
        let global = self.bytes_data(id)?;
        self.initialize_pointer(init, offset, Target::Global(global), 0)?;
        if wide {
            init.bits(offset + 8, 8, self.constants.bytes(id)?.len() as u128)?;
        }
        Ok(())
    }
    fn initialize_pointer(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        target: Target,
        addend: i64,
    ) -> Result<(), &'static str> {
        init.pad(offset)?;
        if init.align >= 8 && offset.is_multiple_of(8) {
            init.items.push(Item::Address { target, addend });
            init.cursor += 8;
        } else {
            let storage = &mut self.output.globals[init.global.0 as usize].storage;
            if *storage == GlobalStorage::ThreadLocal {
                return Err("thread-local data cannot hold an unaligned address");
            }
            *storage = GlobalStorage::Writable;
            self.data.fixups.push(Fixup {
                global: init.global,
                offset,
                target,
                addend,
            });
            init.zero(8)?;
        }
        Ok(())
    }
}
impl Initializer {
    fn zero(&mut self, bytes: u64) -> Result<(), &'static str> {
        if bytes != 0 {
            if let Some(Item::Zero(last)) = self.items.last_mut() {
                *last = last
                    .checked_add(bytes)
                    .ok_or("constant zero run overflow")?;
            } else {
                self.items.push(Item::Zero(bytes));
            }
            self.cursor = self
                .cursor
                .checked_add(bytes)
                .ok_or("constant data cursor overflow")?;
        }
        Ok(())
    }
    /// `length` bytes from `offset`: a run of at least [`super::WORD_RUN`]
    /// bytes in storage aligned to eight takes one item per aligned 8-byte
    /// word, the rest one item per byte (zero bytes still merge into runs).
    fn byte_run(
        &mut self,
        offset: u64,
        length: usize,
        byte: impl Fn(usize) -> u8,
    ) -> Result<(), &'static str> {
        let words = length as u64 >= super::WORD_RUN && self.align >= 8;
        let mut index = 0;
        while index < length {
            let at = offset + index as u64;
            if words && at.is_multiple_of(8) && length - index >= 8 {
                let word = (0..8).fold(0_u64, |word, k| {
                    word | (u64::from(byte(index + k)) << (8 * k))
                });
                self.bits(at, 8, u128::from(word))?;
                index += 8;
            } else {
                self.bits(at, 1, u128::from(byte(index)))?;
                index += 1;
            }
        }
        Ok(())
    }
    fn pad(&mut self, offset: u64) -> Result<(), &'static str> {
        self.zero(
            offset
                .checked_sub(self.cursor)
                .ok_or("constant data overlaps an earlier field")?,
        )
    }
    fn bits(&mut self, offset: u64, bytes: u64, bits: u128) -> Result<(), &'static str> {
        let ty = match bytes {
            1 => Type::U8,
            2 => Type::U16,
            4 => Type::U32,
            8 => Type::I64,
            16 => Type::I128,
            _ => return Err("constant scalar has an invalid byte width"),
        };
        if bytes < 16 && bits >> (bytes * 8) != 0 {
            return Err("constant bits exceed their storage width");
        }
        self.pad(offset)?;
        if bits == 0 {
            return self.zero(bytes);
        }
        if u64::from(self.align) >= bytes && offset.is_multiple_of(bytes) {
            self.items.push(Item::Bits { ty, bits });
            self.cursor += bytes;
        } else {
            let width =
                usize::try_from(bytes).map_err(|_| "constant scalar width exceeds host size")?;
            for byte in &bits.to_le_bytes()[..width] {
                if *byte == 0 {
                    self.zero(1)?;
                } else {
                    self.items.push(Item::Bits {
                        ty: Type::U8,
                        bits: u128::from(*byte),
                    });
                    self.cursor += 1;
                }
            }
        }
        Ok(())
    }
}
pub(super) const fn bits(scalar: Scalar) -> u128 {
    match scalar {
        Scalar::Integer(bits) => bits,
        Scalar::Boolean(value) => value as u128,
        Scalar::Character(value) => value as u128,
        Scalar::Float(FloatBits::F32(bits)) => bits as u128,
        Scalar::Float(FloatBits::F64(bits)) => bits as u128,
        Scalar::Float(FloatBits::F16(bits)) => bits as u128,
        Scalar::Float(FloatBits::F128 { low, high }) => low as u128 | ((high as u128) << 64),
    }
}
