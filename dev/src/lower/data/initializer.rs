//! Serialize completed constants in canonical byte order, with bounded flat work.
mod images;
use super::{Context, Fixup, Key, Target, Type};
use crate::{
    abi::layout::{Layout, Shape},
    literal::value::FloatBits,
    lowir::data::Item,
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
                let symbol = self.demand(id)?;
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
                self.data.charge(
                    usize::try_from(length).map_err(|_| "constant product exceeds host size")?,
                )?;
                for index in (0..length).rev() {
                    let (delta, ty) = self.constant_field(&layout, Some(value), index)?;
                    init.work.push(Work::Value {
                        offset: offset + delta,
                        ty,
                        value: self.constants.field(value, index)?,
                    });
                }
                if let Some(niche) = self.layouts.niche(&layout)? {
                    let ordinal = self.constant_variant(value)?;
                    if ordinal == niche.empty as usize && length == 0 {
                        init.work.push(Work::Bits {
                            offset: offset + niche.offset,
                            bytes: 8,
                            bits: 0,
                        });
                    } else if ordinal != niche.payload as usize || length != 1 {
                        return Err("nullable constant differs from its canonical variant");
                    }
                }
                if let Shape::Aggregate { tag: Some(tag), .. } = layout.shape {
                    let variant = self
                        .layouts
                        .variants(&layout)
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
        }
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
        for (index, &byte) in bytes.iter().enumerate() {
            init.bits(offset + index as u64, 1, u128::from(byte))?;
        }
        Ok(())
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
        self.data
            .charge(usize::try_from(length).map_err(|_| "constant repeat exceeds host size")?)?;
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
        let size = (bytes.len() as u64).max(1);
        let global = self.output.global(
            self.internal_name(format!("rg.data.{}", self.output.globals.len())),
            crate::lowir::program::MemorySpan {
                bytes: size,
                align: 1,
            },
            true,
        )?;
        let mut init = Initializer {
            global,
            items: Vec::new(),
            cursor: 0,
            align: 1,
            work: Vec::new(),
        };
        for (i, &byte) in bytes.iter().enumerate() {
            init.bits(i as u64, 1, u128::from(byte))?;
        }
        init.pad(size)?;
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
            self.output.globals[init.global.0 as usize].readonly = false;
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
    }
}
