//! Wide pointer values use object storage; unsized places retain sparse length facts.
use super::{Address, Arithmetic, Lower, Projected, Selected};
use crate::{
    body::ir::{Place, ProjectionId, ValueId},
    lowir::{
        program::{MemorySpan, Value},
        vocabulary::{Predicate, Type},
    },
    types::{Kind, Primitive, TypeId},
};

impl Lower<'_, '_> {
    pub(super) fn text_literal(
        &mut self,
        text: crate::sema::text::Id,
        representation: super::Representation,
    ) -> Result<Selected, &'static str> {
        let global = self.context.bytes_data(text)?;
        let pointer = self
            .builder
            .address(crate::lowir::program::Operand::Global(global))?;
        if matches!(representation, super::Representation::Object(_)) {
            let length = self.context.constants.bytes(text)?.len() as u128;
            let length = self.builder.integer(Type::I64, length)?;
            self.wide_pointer(pointer, length)
        } else {
            Ok(Selected::Runtime(pointer))
        }
    }
    pub(super) fn wide_pointer(
        &mut self,
        pointer: Value,
        length: Value,
    ) -> Result<Selected, &'static str> {
        if pointer.ty != Type::Ptr || length.ty != Type::I64 {
            return Err("wide pointer components have invalid types");
        }
        let slot = self.builder.object_slot(MemorySpan {
            bytes: 16,
            align: 8,
        })?;
        let address = Address {
            operand: crate::lowir::program::Operand::Slot(slot),
            align: 8,
        };
        self.builder.store(pointer, address.operand)?;
        let tail = self.offset(address, 8)?;
        self.builder.store(length, tail.operand)?;
        Ok(Selected::Object(address))
    }
    pub(super) fn unsize(
        &mut self,
        operand: ValueId,
        target: TypeId,
    ) -> Result<Selected, &'static str> {
        let source = self.body.values[operand.0 as usize].ty;
        let (
            Some(Kind::Reference {
                referent: source, ..
            }),
            Some(Kind::Reference {
                referent: target, ..
            }),
        ) = (
            self.context.types.kind(source),
            self.context.types.kind(target),
        )
        else {
            return Err("reference conversion changes its runtime representation");
        };
        let (Some(Kind::Array { element, length }), Some(Kind::Slice(target))) = (
            self.context.types.kind(source),
            self.context.types.kind(target),
        ) else {
            return Err("unsizing requires an array reference and slice target");
        };
        let from = self.context.layout(element)?;
        let to = self.context.layout(target)?;
        if from.size != to.size || from.align != to.align {
            return Err("slice conversion changes its element representation");
        }
        let pointer = self.value(operand)?.runtime()?;
        let length = self.builder.integer(Type::I64, u128::from(length))?;
        self.wide_pointer(pointer, length)
    }
    pub(super) fn slice_length(&mut self, place: Place) -> Result<Value, &'static str> {
        if let Some(Kind::Array { length, .. }) = self.context.types.kind(self.place_type(place)?) {
            return self.builder.integer(Type::I64, u128::from(length));
        }
        let Place::Projection(id) = place else {
            return Err("unsized place lacks projection metadata");
        };
        self.slice_lengths
            .get(&id)
            .copied()
            .ok_or("slice length does not dominate its use")
    }
    pub(super) fn project_reference(
        &mut self,
        id: ProjectionId,
        base: Address,
    ) -> Result<(), &'static str> {
        let ty = self.body.projections[id.0 as usize].ty;
        let address = self.dereference(base, ty)?;
        if matches!(
            self.context.types.kind(ty),
            Some(Kind::Slice(_) | Kind::Primitive(Primitive::Str))
        ) {
            let tail = self.offset(base, 8)?;
            let length = self.builder.load(Type::I64, tail.operand)?;
            self.slice_lengths.insert(id, length);
        }
        self.projections[id.0 as usize] = Projected::Ready(Some(address));
        Ok(())
    }
    pub(super) fn project_slice_index(
        &mut self,
        id: ProjectionId,
        index: ValueId,
    ) -> Result<(), &'static str> {
        let projection = &self.body.projections[id.0 as usize];
        let place = projection.base;
        let Some(Kind::Slice(element)) = self.context.types.kind(self.place_type(place)?) else {
            return Err("dynamic index requires a slice place");
        };
        if element != projection.ty {
            return Err("slice index differs from its element type");
        }
        let leaf = self.context.layout(element)?;
        let index = self.value(index)?.runtime()?;
        let limit = self.slice_length(place)?;
        let fault = self.builder.compare(Predicate::Uge, index, limit)?;
        Arithmetic {
            builder: &mut self.builder,
            panic: &mut self.panic,
        }
        .guard(fault)?;
        let address = if leaf.size != 0 || self.addresses.needs(Place::Projection(id)) {
            let base = self.place(place)?.ok_or("slice data has no address")?;
            Some(self.indexed(base, index, leaf.size, leaf.align)?)
        } else {
            None
        };
        self.projections[id.0 as usize] = Projected::Ready(address);
        Ok(())
    }
}
