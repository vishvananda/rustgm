//! Owned snapshots and address projections use completed layouts and selected fields.
use super::{Address, Arithmetic, Lower, Projected, Representation, Selected};
use crate::{
    abi::layout::{Layout, Shape},
    body::ir::{Place, ProjectionId, ProjectionKind, Selection, ValueId, ValueList},
    lowir::{
        program::{MemorySpan, Operand, Terminator, Value},
        vocabulary::{Binary, Predicate, Type},
    },
    sema::members::VariantId,
    types::TypeId,
};

impl Lower<'_, '_> {
    pub(super) fn is_variant(
        &mut self,
        place: Place,
        variant: VariantId,
    ) -> Result<Selected, &'static str> {
        let layout = self.context.layout(self.place_type(place)?)?;
        let selected = self
            .context
            .members
            .variants
            .get(variant.0 as usize)
            .ok_or("missing tested variant")?;
        let discriminant = self
            .context
            .layouts
            .variants(&layout)
            .get(selected.ordinal as usize)
            .ok_or("missing tested variant layout")?
            .discriminant;
        let Shape::Aggregate { tag: Some(tag), .. } = layout.shape else {
            return Err("constructor decision has no physical tag");
        };
        let base = self
            .place(place)?
            .ok_or("constructor test has no storage")?;
        let address = self.offset(base, tag.offset)?;
        let ty =
            super::super::context::representation(self.context.types, TypeId::primitive(tag.ty))?;
        let input = self.builder.load(ty, address.operand)?;
        let expected = self.builder.integer(ty, discriminant)?;
        self.builder
            .compare(Predicate::Eq, input, expected)
            .map(Selected::Runtime)
    }
    pub(super) fn enum_cast(
        &mut self,
        operand: ValueId,
        target: Type,
    ) -> Result<Selected, &'static str> {
        let layout = self
            .context
            .layout(self.body.values[operand.0 as usize].ty)?;
        let (value, signed) = if let Shape::Aggregate { tag: Some(tag), .. } = layout.shape {
            let Selected::Object(address) = self.value(operand)? else {
                return Err("enum tag has no storage");
            };
            let address = self.offset(address, tag.offset)?;
            let ty = super::super::context::representation(
                self.context.types,
                TypeId::primitive(tag.ty),
            )?;
            (
                self.builder.load(ty, address.operand)?,
                tag.ty.integer().is_some_and(|(signed, _)| signed),
            )
        } else {
            // A single-variant enum has a fixed discriminant without a physical
            // tag. An empty enum has no inhabitants that can reach this operation.
            let bits = self
                .context
                .layouts
                .variants(&layout)
                .first()
                .map_or(0, |v| v.discriminant);
            (self.builder.integer(Type::I64, bits)?, true)
        };
        super::super::casts::integer(&mut self.builder, value, target, signed)
            .map(Selected::Runtime)
    }
    pub(super) fn allocate_local(
        &mut self,
        ty: TypeId,
        kind: crate::body::ir::LocalKind,
        addressed: bool,
    ) -> Result<Option<Address>, &'static str> {
        if !matches!(kind, crate::body::ir::LocalKind::Guard(_)) {
            let address = self.allocate(ty)?;
            return if address.is_none() && addressed {
                self.zero_backing(ty).map(Some)
            } else {
                Ok(address)
            };
        }
        if self.context.layout(ty)?.size == 0 && !addressed {
            return Ok(None);
        }
        Ok(Some(Address {
            operand: Operand::Slot(self.builder.slot(Type::Ptr)?),
            align: 8,
        }))
    }
    pub(super) fn bind_view(
        &mut self,
        view: crate::body::ir::LocalId,
        place: Place,
    ) -> Result<(), &'static str> {
        let Some(destination) = self.locals[view.0 as usize] else {
            return Ok(());
        };
        let address = self
            .place(place)?
            .ok_or("guard view has no source storage")?;
        let source = self.pointer(address)?;
        self.builder.store(source, destination.operand)
    }
    pub(super) fn allocate(&mut self, ty: TypeId) -> Result<Option<Address>, &'static str> {
        let (slot, align) = match self.context.representation(ty)? {
            Representation::Scalar(Type::Void) => return Ok(None),
            Representation::Scalar(ty) => (self.builder.slot(ty)?, ty.bits() / 8),
            Representation::Object(span) => (self.builder.object_slot(span)?, span.align),
        };
        Ok(Some(Address {
            operand: Operand::Slot(slot),
            align,
        }))
    }
    pub(super) fn pointer(&mut self, address: Address) -> Result<Value, &'static str> {
        address.pointer(&mut self.builder)
    }

    pub(super) fn write(
        &mut self,
        destination: Option<Address>,
        selected: Selected,
        ty: TypeId,
    ) -> Result<(), &'static str> {
        match selected {
            Selected::Runtime(value) => self
                .builder
                .store(value, destination.ok_or("missing scalar storage")?.operand),
            Selected::Object(source) => {
                let destination = destination.ok_or("missing object storage")?;
                let Representation::Object(span) = self.context.representation(ty)? else {
                    return Err("object value has no object representation");
                };
                let span = MemorySpan {
                    bytes: span.bytes,
                    align: source.align.min(destination.align),
                };
                let source = self.pointer(source)?;
                let destination = self.pointer(destination)?;
                // Body reads and constructors create distinct snapshots. Arguments
                // copy into fresh callee slots; hidden results target fresh caller
                // slots. No transfer is a potentially overlapping source-place copy.
                self.builder.copy_object(span, source, destination)
            }
            Selected::Unit | Selected::Item(_) => Ok(()),
        }
    }
    pub(super) fn offset(&mut self, base: Address, offset: u64) -> Result<Address, &'static str> {
        base.offset(&mut self.builder, offset)
    }
    pub(super) fn indexed(
        &mut self,
        base: Address,
        index: Value,
        stride: u64,
        align: u32,
    ) -> Result<Address, &'static str> {
        base.indexed(&mut self.builder, index, stride, align)
    }
    pub(super) fn place_type(&self, place: Place) -> Result<TypeId, &'static str> {
        match place {
            Place::Local(id) => Ok(self.body.locals[id.0 as usize].ty),
            Place::Temporary(id) => Ok(self.body.values[id.0 as usize].ty),
            Place::Projection(id) => Ok(self.body.projections[id.0 as usize].ty),
            Place::Static(id) => self
                .context
                .definitions
                .get(id.0 as usize)
                .copied()
                .flatten()
                .ok_or("missing static place type"),
        }
    }
    pub(super) fn project(&mut self, id: ProjectionId) -> Result<(), &'static str> {
        let projection = &self.body.projections[id.0 as usize];
        let base = self.place(projection.base)?;
        if projection.kind.is_dereference() {
            return self
                .project_reference(id, base.ok_or("reference place has no pointer storage")?);
        }
        if let ProjectionKind::SliceIndex { index } = projection.kind {
            return self.project_slice_index(id, index);
        }
        if matches!(projection.kind, ProjectionKind::CaptureBoundary { .. }) {
            if self.place_type(projection.base)? != projection.ty {
                return Err("capture boundary changes native place type");
            }
            if self.context.wide_referent(projection.ty)? {
                let metadata = self.metadata(projection.base)?;
                self.place_metadata.insert(id, metadata);
            }
            self.projections[id.0 as usize] = Projected::Ready(base);
            return Ok(());
        }
        let base_type = self.place_type(projection.base)?;
        if matches!(
            self.context.types.kind(base_type),
            Some(crate::types::Kind::Slice(_))
        ) {
            return self
                .project_slice_pattern(id, base.ok_or("slice pattern has no data address")?);
        }
        if let ProjectionKind::Field(field) | ProjectionKind::UnionField(field) = projection.kind {
            return self.project_field(id, base, base_type, field);
        }
        let layout = self.context.layout(base_type)?;
        let leaf = self.context.layout(projection.ty)?;
        let retained = leaf.size != 0 || self.addresses.needs(Place::Projection(id));
        let address = match projection.kind {
            ProjectionKind::CaptureBoundary { .. } => {
                return Err("capture boundary escaped transparent native dispatch");
            }
            ProjectionKind::Dereference { .. }
            | ProjectionKind::RawDereference { .. }
            | ProjectionKind::SliceIndex { .. }
            | ProjectionKind::FromEnd { .. }
            | ProjectionKind::Subslice { .. } => {
                return Err("reference projection escaped its typed dispatch");
            }
            ProjectionKind::Field(_) | ProjectionKind::UnionField(_) => {
                return Err("field projection escaped its typed dispatch");
            }
            ProjectionKind::IndexParameter { .. } => {
                return Err("symbolic array bound requires concrete instantiation");
            }
            ProjectionKind::Index { index, length } => {
                let index = self.value(index)?.runtime()?;
                let limit = self.builder.integer(Type::I64, u128::from(length))?;
                let fault = self.builder.compare(Predicate::Uge, index, limit)?;
                Arithmetic {
                    builder: &mut self.builder,
                    panic: &mut self.panic,
                }
                .guard(fault)?;
                let Shape::Array {
                    stride,
                    length: actual,
                    ..
                } = layout.shape
                else {
                    return Err("indexed base has no array layout");
                };
                if actual != length {
                    return Err("indexed bound differs from canonical layout");
                }
                if retained {
                    let base = match base {
                        Some(base) => base,
                        // The guard always panics for an empty array. Its
                        // unreachable continuation still needs a typed address.
                        None if length == 0 => self.zero_backing(base_type)?,
                        None => return Err("nonzero array has no storage"),
                    };
                    Some(self.indexed(base, index, stride, leaf.align)?)
                } else {
                    None
                }
            }
            ProjectionKind::ConstantIndex(index) => {
                self.pattern_offset(base, &layout, &leaf, index, None, retained)?
            }
            ProjectionKind::Subarray { start, end } => {
                self.pattern_offset(base, &layout, &leaf, start, Some(end), retained)?
            }
        };
        self.projections[id.0 as usize] = Projected::Ready(address);
        Ok(())
    }
    fn project_field(
        &mut self,
        id: ProjectionId,
        base: Option<Address>,
        base_type: TypeId,
        field: Selection,
    ) -> Result<(), &'static str> {
        let projection = &self.body.projections[id.0 as usize];
        let (_, offset) = self.context.place_field(base_type, field)?;
        let ty = projection.ty;
        let wide = self.context.wide_referent(ty)?;
        let metadata = if self.context.wide_referent(base_type)? {
            Some(self.metadata(projection.base)?)
        } else {
            None
        };
        let retained = wide
            || self.context.layout(ty)?.size != 0
            || self.addresses.needs(Place::Projection(id));
        let address = if retained {
            Some(self.field_address(
                base.ok_or("retained field has no base storage")?,
                offset,
                metadata,
            )?)
        } else {
            None
        };
        if wide {
            self.place_metadata
                .insert(id, metadata.ok_or("unsized field lacks metadata")?);
        }
        self.projections[id.0 as usize] = Projected::Ready(address);
        Ok(())
    }
    fn pattern_offset(
        &mut self,
        base: Option<Address>,
        layout: &Layout,
        leaf: &Layout,
        start: u64,
        end: Option<u64>,
        retained: bool,
    ) -> Result<Option<Address>, &'static str> {
        let offset = Self::pattern_byte_offset(layout, leaf, start, end)?;
        if !retained {
            return Ok(None);
        }
        self.offset(base.ok_or("pattern array has no storage")?, offset)
            .map(Some)
    }
    pub(super) fn pattern_byte_offset(
        layout: &Layout,
        leaf: &Layout,
        start: u64,
        end: Option<u64>,
    ) -> Result<u64, &'static str> {
        let Shape::Array { stride, length, .. } = layout.shape else {
            return Err("pattern projection base has no array layout");
        };
        if let Some(end) = end {
            if start > end || end > length {
                return Err("pattern subarray exceeds its checked bound");
            }
            let Shape::Array {
                length: actual,
                stride: leaf_stride,
                ..
            } = leaf.shape
            else {
                return Err("pattern subarray has no array layout");
            };
            if actual != end - start || leaf_stride != stride {
                return Err("pattern subarray layout differs from selection");
            }
        } else if start >= length {
            return Err("pattern element exceeds its checked bound");
        }
        start
            .checked_mul(stride)
            .ok_or("pattern array offset overflow")
    }
    pub(super) fn union(
        &mut self,
        ty: TypeId,
        field: Selection,
        value: ValueId,
    ) -> Result<Selected, &'static str> {
        let Some(address) = self.allocate(ty)? else {
            return Ok(Selected::Unit);
        };
        let (_, offset) = self.context.place_field(ty, field)?;
        let destination = self.field_address(address, offset, None)?;
        self.write(
            Some(destination),
            self.value(value)?,
            self.body.values[value.0 as usize].ty,
        )?;
        Ok(Selected::Object(address))
    }
    pub(super) fn aggregate(
        &mut self,
        ty: TypeId,
        variant: Option<VariantId>,
        fields: ValueList,
    ) -> Result<Selected, &'static str> {
        let Some(address) = self.allocate(ty)? else {
            return Ok(Selected::Unit);
        };
        let layout = self.context.layout(ty)?;
        let ordinal = variant.map_or(Ok(0), |id| {
            self.context
                .members
                .variants
                .get(id.0 as usize)
                .map(|v| v.ordinal)
                .ok_or("missing constructed variant")
        })?;
        if let Shape::Aggregate { tag: Some(tag), .. } = layout.shape {
            let variant = self
                .context
                .layouts
                .variants(&layout)
                .get(ordinal as usize)
                .ok_or("missing constructed variant layout")?;
            let bits = variant.discriminant;
            let tag_type = super::super::context::representation(
                self.context.types,
                TypeId::primitive(tag.ty),
            )?;
            let value = self.builder.integer(tag_type, bits)?;
            let destination = self.offset(address, tag.offset)?;
            self.builder.store(value, destination.operand)?;
        }
        for (index, id) in self.body.arguments(fields).iter().copied().enumerate() {
            let offset = self.field_offset(&layout, ordinal, index)?;
            let destination = self.offset(address, offset)?;
            self.write(
                Some(destination),
                self.value(id)?,
                self.body.values[id.0 as usize].ty,
            )?;
        }
        Ok(Selected::Object(address))
    }
    fn field_offset(
        &self,
        layout: &Layout,
        ordinal: u32,
        field: usize,
    ) -> Result<u64, &'static str> {
        match layout.shape {
            Shape::Array { stride, length, .. } => {
                let index = u64::try_from(field).map_err(|_| "array field index exceeds target")?;
                if index >= length {
                    return Err("too many array initializer elements");
                }
                index
                    .checked_mul(stride)
                    .ok_or("array field offset overflow")
            }
            Shape::Aggregate { .. } => {
                let variant = self
                    .context
                    .layouts
                    .variants(layout)
                    .get(ordinal as usize)
                    .ok_or("missing aggregate field group")?;
                self.context
                    .layouts
                    .offsets(variant)
                    .get(field)
                    .copied()
                    .ok_or("missing aggregate initializer offset")
            }
            Shape::Scalar | Shape::WidePointer => Err("aggregate initializer has scalar layout"),
        }
    }
    pub(super) fn repeat(
        &mut self,
        ty: TypeId,
        operand: ValueId,
    ) -> Result<Selected, &'static str> {
        let Some(address) = self.allocate(ty)? else {
            return Ok(Selected::Unit);
        };
        let Shape::Array {
            element,
            stride,
            length,
        } = self.context.layout(ty)?.shape
        else {
            return Err("repeat has no array layout");
        };
        let selected = self.value(operand)?;
        if length == 1 {
            self.write(Some(address), selected, element)?;
            return Ok(Selected::Object(address));
        }
        // The source is one evaluated Copy snapshot. Emit a constant-size loop,
        // independent of the array bound; zero-size arrays took the path above.
        let counter = self.builder.slot(Type::I64)?;
        let zero = self.builder.integer(Type::I64, 0)?;
        self.builder.store(zero, Operand::Slot(counter))?;
        let header = self.builder.block()?;
        let body = self.builder.block()?;
        let done = self.builder.block()?;
        self.builder.terminate(Terminator::Jump(header))?;
        self.builder.enter(header)?;
        let index = self.builder.load(Type::I64, Operand::Slot(counter))?;
        let limit = self.builder.integer(Type::I64, u128::from(length))?;
        let condition = self.builder.compare(Predicate::Ult, index, limit)?;
        self.builder.terminate(Terminator::Branch {
            condition: condition.operand,
            yes: body,
            no: done,
        })?;
        self.builder.enter(body)?;
        let align = self.context.layout(element)?.align;
        let destination = self.indexed(address, index, stride, align)?;
        self.write(Some(destination), selected, element)?;
        let one = self.builder.integer(Type::I64, 1)?;
        let next = self.builder.binary(Binary::Add, index, one)?;
        self.builder.store(next, Operand::Slot(counter))?;
        self.builder.terminate(Terminator::Jump(header))?;
        self.builder.enter(done)?;
        Ok(Selected::Object(address))
    }
}
