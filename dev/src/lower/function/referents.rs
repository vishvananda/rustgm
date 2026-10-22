//! Typed metadata and runtime alignment for projections through unsized tails.
use super::{Address, Lower};
use crate::{
    abi::{
        metadata::Metadata,
        tails::{Alignment, Offset},
    },
    body::ir::Place,
    lowir::{
        program::Value,
        vocabulary::{Binary, Convert, Predicate, Type, Unary},
    },
    types::TypeId,
};

impl Lower<'_, '_, '_> {
    pub(super) fn metadata(&mut self, place: Place) -> Result<Value, &'static str> {
        // A selected reborrow can form a slice directly from array storage,
        // without an intermediate wide pointer or a metadata-bearing projection.
        if let Some(crate::types::Kind::Array { length, .. }) =
            self.context.types.kind(self.place_type(place)?)
        {
            return self.builder.integer(Type::I64, u128::from(length));
        }
        let Place::Projection(id) = place else {
            return Err("unsized place lacks projection metadata");
        };
        self.place_metadata
            .get(id)
            .copied()
            .ok_or("place metadata does not dominate its use")
    }
    pub(super) fn load_metadata(
        &mut self,
        base: Address,
        ty: TypeId,
    ) -> Result<Option<Value>, &'static str> {
        let ty = match self.context.layouts.pointer_metadata(
            self.context.types,
            self.context.aggregates,
            ty,
        )? {
            Metadata::Thin => return Ok(None),
            Metadata::Length => Type::I64,
            Metadata::VTable(_) => Type::Ptr,
            Metadata::Unknown(_) => return Err("referent has unresolved metadata"),
        };
        let tail = self.offset(base, 8)?;
        self.builder.load(ty, tail.operand).map(Some)
    }
    pub(super) fn field_address(
        &mut self,
        base: Address,
        offset: Offset,
        metadata: Option<Value>,
    ) -> Result<Address, &'static str> {
        let (prefix, alignment) = match offset {
            Offset::Fixed(offset) => return self.offset(base, offset),
            Offset::Aligned { prefix, alignment } => (prefix, alignment),
        };
        let table = metadata.ok_or("dynamic field offset lacks object metadata")?;
        if table.ty != Type::Ptr {
            return Err("dynamic field offset requires vtable metadata");
        }
        let address = self.offset(
            Address {
                operand: table.operand,
                align: 8,
            },
            16,
        )?;
        let value = self.builder.load(Type::I64, address.operand)?;
        let value = self.clamp_alignment(value, alignment)?;
        let one = self.builder.integer(Type::I64, 1)?;
        let mask = self.builder.binary(Binary::Sub, value, one)?;
        let prefix = self.builder.integer(Type::I64, u128::from(prefix))?;
        let rounded = self.builder.binary(Binary::Add, prefix, mask)?;
        let mask = self.builder.unary(Unary::BitNot, mask)?;
        let offset = self.builder.binary(Binary::And, rounded, mask)?;
        let pointer = self.pointer(base)?;
        let operand = self.builder.index(Type::U8, pointer, offset)?.operand;
        Ok(Address {
            operand,
            align: base.align.min(alignment.minimum),
        })
    }
    fn clamp_alignment(
        &mut self,
        mut value: Value,
        alignment: Alignment,
    ) -> Result<Value, &'static str> {
        if alignment.minimum != 1 {
            value = self.alignment_limit(value, alignment.minimum, Predicate::Ult)?;
        }
        if alignment.maximum != Alignment::OBJECT.maximum {
            value = self.alignment_limit(value, alignment.maximum, Predicate::Ugt)?;
        }
        Ok(value)
    }
    fn alignment_limit(
        &mut self,
        value: Value,
        bound: u32,
        predicate: Predicate,
    ) -> Result<Value, &'static str> {
        let bound = self.builder.integer(Type::I64, u128::from(bound))?;
        let outside = self.builder.compare(predicate, value, bound)?;
        let outside = self.builder.convert(Convert::Zext, Type::I64, outside)?;
        let zero = self.builder.integer(Type::I64, 0)?;
        let mask = self.builder.binary(Binary::Sub, zero, outside)?;
        let difference = self.builder.binary(Binary::Xor, value, bound)?;
        let difference = self.builder.binary(Binary::And, difference, mask)?;
        self.builder.binary(Binary::Xor, value, difference)
    }
}
