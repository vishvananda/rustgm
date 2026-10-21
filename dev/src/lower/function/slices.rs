//! Wide pointer values use object storage; slice projections consume length metadata.
use super::{Address, Arithmetic, Lower, Projected, Selected};
use crate::{
    body::ir::{Place, ProjectionId, ProjectionKind, ValueId},
    lowir::{
        program::{MemorySpan, Value},
        vocabulary::{Binary, Predicate, Type},
    },
    types::{Kind, TypeId},
};

impl Lower<'_, '_> {
    pub(super) fn dynamic_selection(
        &mut self,
        id: ProjectionId,
        base: Address,
        metadata: Option<Value>,
    ) -> Result<(Address, Option<Value>), &'static str> {
        let projection = &self.body.projections[id.0 as usize];
        if projection.kind.is_dereference() {
            let address = self.dereference(base, projection.ty)?;
            return Ok((address, self.load_metadata(base, projection.ty)?));
        }
        if let ProjectionKind::Field(field) | ProjectionKind::UnionField(field) = projection.kind {
            let (_, offset) = self
                .context
                .place_field(self.place_type(projection.base)?, field)?;
            let address = self.field_address(base, offset, metadata)?;
            return Ok((address, metadata));
        }
        let Some(Kind::Slice(element)) = self.context.types.kind(self.place_type(projection.base)?)
        else {
            return Err("dynamic pattern selection requires a slice");
        };
        let leaf = self.context.layout(element)?;
        let length = metadata.ok_or("dynamic selection lacks slice length")?;
        if length.ty != Type::I64 {
            return Err("slice selection requires length metadata");
        }
        match projection.kind {
            ProjectionKind::FromEnd { offset, minimum } if offset != 0 && offset <= minimum => {
                let offset = self.builder.integer(Type::I64, u128::from(offset))?;
                let index = self.builder.binary(Binary::Sub, length, offset)?;
                Ok((self.indexed(base, index, leaf.size, leaf.align)?, None))
            }
            ProjectionKind::Subslice { prefix, suffix } => {
                let excluded = prefix
                    .checked_add(suffix)
                    .ok_or("slice pattern length overflow")?;
                let excluded = self.builder.integer(Type::I64, u128::from(excluded))?;
                let length = self.builder.binary(Binary::Sub, length, excluded)?;
                let offset = prefix
                    .checked_mul(leaf.size)
                    .ok_or("slice pattern offset overflow")?;
                Ok((self.offset(base, offset)?, Some(length)))
            }
            _ => Err("invalid dynamic pattern projection"),
        }
    }
    pub(super) fn project_slice_pattern(
        &mut self,
        id: ProjectionId,
        base: Address,
    ) -> Result<(), &'static str> {
        let projection = &self.body.projections[id.0 as usize];
        let (address, length) = if let ProjectionKind::ConstantIndex(index) = projection.kind {
            let leaf = self.context.layout(projection.ty)?;
            let offset = index
                .checked_mul(leaf.size)
                .ok_or("slice pattern offset overflow")?;
            (self.offset(base, offset)?, None)
        } else {
            let length = self.slice_length(projection.base)?;
            self.dynamic_selection(id, base, Some(length))?
        };
        if let Some(length) = length {
            self.place_metadata.insert(id, length);
        }
        self.projections[id.0 as usize] = Projected::Ready(Some(address));
        Ok(())
    }
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
        metadata: Value,
    ) -> Result<Selected, &'static str> {
        if pointer.ty != Type::Ptr || !matches!(metadata.ty, Type::I64 | Type::Ptr) {
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
        self.builder.store(metadata, tail.operand)?;
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
    pub(super) fn selected_unsizing(
        &mut self,
        operand: ValueId,
        array: TypeId,
    ) -> Result<Selected, &'static str> {
        let Some(Kind::Array { length, .. }) = self.context.types.kind(array) else {
            return Err("selected unsizing retains an unresolved terminal array");
        };
        let pointer = self.value(operand)?.runtime()?;
        let length = self.builder.integer(Type::I64, u128::from(length))?;
        self.wide_pointer(pointer, length)
    }
    pub(super) fn object_unsizing(
        &mut self,
        operand: ValueId,
        concrete: TypeId,
        object: TypeId,
    ) -> Result<Selected, &'static str> {
        let pointer = self.value(operand)?.runtime()?;
        let table = self.context.object_table(concrete, object)?;
        let metadata = self
            .builder
            .address(crate::lowir::program::Operand::Global(table))?;
        self.wide_pointer(pointer, metadata)
    }
    pub(super) fn object_upcasting(
        &mut self,
        operand: ValueId,
        source: TypeId,
        target: TypeId,
    ) -> Result<Selected, &'static str> {
        use crate::sema::vtable_plans::Upcast;
        let plan = self
            .context
            .objects
            .upcast(source, target)
            .ok_or("object upcast lacks a checked plan")?;
        let value = self.value(operand)?;
        let Upcast::Table { source, index } = plan else {
            return Ok(value);
        };
        let Selected::Object(address) = value else {
            return Err("upcast source lacks its wide pointer representation");
        };
        let shape = self
            .context
            .objects
            .shape(source)
            .ok_or("missing upcast source shape")?;
        let slot = u64::try_from(shape.slots.len())
            .map_err(|_| "virtual slot count exceeds target address space")?
            .checked_add(u64::from(index))
            .and_then(|n| n.checked_mul(8))
            .and_then(|n| n.checked_add(super::super::virtual_dispatch::HEADER_BYTES))
            .ok_or("upcast slot exceeds target address space")?;
        let data = self.builder.load(Type::Ptr, address.operand)?;
        let metadata = self.offset(address, 8)?;
        let table = self.builder.load(Type::Ptr, metadata.operand)?;
        let slot = self.offset(
            Address {
                operand: table.operand,
                align: 8,
            },
            slot,
        )?;
        let metadata = self.builder.load(Type::Ptr, slot.operand)?;
        self.wide_pointer(data, metadata)
    }
    pub(super) fn slice_length(&mut self, place: Place) -> Result<Value, &'static str> {
        let value = self.metadata(place)?;
        if value.ty != Type::I64 {
            return Err("slice length requires length metadata");
        }
        Ok(value)
    }
    pub(super) fn project_reference(
        &mut self,
        id: ProjectionId,
        base: Address,
    ) -> Result<(), &'static str> {
        let ty = self.body.projections[id.0 as usize].ty;
        let address = self.dereference(base, ty)?;
        if let Some(metadata) = self.load_metadata(base, ty)? {
            self.place_metadata.insert(id, metadata);
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
