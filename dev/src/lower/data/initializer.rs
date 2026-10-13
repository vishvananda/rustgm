//! Serialize completed constants in canonical byte order, with bounded flat work.
use super::{Context, Fixup, Key, Pending, Target, Type};
use crate::{
    abi::layout::Shape,
    literal::value::FloatBits,
    lowir::data::Item,
    sema::{
        model::Scalar,
        values::{Constant, View},
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
    object: Pending,
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
            object,
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
            View::Function(id) => {
                let symbol = self.demand(id)?;
                self.initialize_pointer(init, offset, Target::Function(symbol), 0)
            }
            View::Reference(id) => {
                if layout.size != 8 {
                    return Err("constant reference requires a thin pointer layout");
                }
                let pointer = self.constant_pointer(id)?;
                let addend = i64::try_from(pointer.offset)
                    .map_err(|_| "constant address offset overflow")?;
                self.initialize_pointer(init, offset, Target::Global(pointer.global), addend)
            }
            View::Bytes(id) => {
                if let Some(Kind::Reference { referent, .. }) = self.types.kind(ty) {
                    let global =
                        self.constant_data(Key::Bytes(id, referent), value, referent, true)?;
                    return self.initialize_pointer(init, offset, Target::Global(global), 0);
                }
                let bytes = self.constants.bytes(id)?;
                if !matches!(layout.shape, Shape::Array { stride: 1, length, .. } if length == bytes.len() as u64)
                {
                    return Err("constant bytes differ from their array layout");
                }
                self.data.charge(bytes.len())?;
                for (index, &byte) in bytes.iter().enumerate() {
                    init.bits(offset + index as u64, 1, u128::from(byte))?;
                }
                Ok(())
            }
            View::Text(_) => Err("text constants require wide reference lowering"),
            View::Repeat {
                value: repeated,
                length,
            } => {
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
                self.data.charge(
                    usize::try_from(length).map_err(|_| "constant repeat exceeds host size")?,
                )?;
                for index in (0..length).rev() {
                    init.work.push(Work::Value {
                        offset: offset + index * stride,
                        ty: element,
                        value: repeated,
                    });
                }
                Ok(())
            }
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
            self.output.globals[init.object.global.0 as usize].readonly = false;
            self.data.fixups.push(Fixup {
                global: init.object.global,
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
