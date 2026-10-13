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
    ) -> Result<Option<Address>, &'static str> {
        if !matches!(kind, crate::body::ir::LocalKind::Guard(_)) {
            return self.allocate(ty);
        }
        if self.context.layout(ty)?.size == 0 {
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
        if matches!(address.operand, Operand::Slot(_) | Operand::Symbol(_)) {
            self.builder.address(address.operand)
        } else {
            Ok(Value {
                ty: Type::Ptr,
                operand: address.operand,
            })
        }
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
        if offset == 0 {
            return Ok(base);
        }
        let pointer = self.pointer(base)?;
        let value = self.builder.integer(Type::I64, u128::from(offset))?;
        let operand = self.builder.index(Type::U8, pointer, value)?.operand;
        let align = base.align.min(1_u32 << offset.trailing_zeros().min(29));
        Ok(Address { operand, align })
    }
    fn indexed(
        &mut self,
        base: Address,
        index: Value,
        stride: u64,
        align: u32,
    ) -> Result<Address, &'static str> {
        let stride = self.builder.integer(Type::I64, u128::from(stride))?;
        let offset = self.builder.binary(Binary::Mul, index, stride)?;
        let base_pointer = self.pointer(base)?;
        let operand = self.builder.index(Type::U8, base_pointer, offset)?.operand;
        Ok(Address {
            operand,
            align: base.align.min(align),
        })
    }
    pub(super) fn place_type(&self, place: Place) -> Result<TypeId, &'static str> {
        match place {
            Place::Local(id) => Ok(self.body.locals[id.0 as usize].ty),
            Place::Temporary(id) => Ok(self.body.values[id.0 as usize].ty),
            Place::Projection(id) => Ok(self.body.projections[id.0 as usize].ty),
            Place::Static(_) => Err("static places require allocation lowering"),
        }
    }
    pub(super) fn project(&mut self, id: ProjectionId) -> Result<(), &'static str> {
        let projection = &self.body.projections[id.0 as usize];
        let base = self.place(projection.base)?;
        let base_type = self.place_type(projection.base)?;
        let layout = self.context.layout(base_type)?;
        let leaf = self.context.layout(projection.ty)?;
        let address = match projection.kind {
            ProjectionKind::Dereference { .. } => {
                return Err("reference lowering requires completed region and loan proofs");
            }
            ProjectionKind::Field(field) => {
                let offset = self.selected_field_offset(&layout, field)?;
                if leaf.size == 0 {
                    None
                } else {
                    Some(self.offset(base.ok_or("nonzero field has no base storage")?, offset)?)
                }
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
                if leaf.size == 0 {
                    None
                } else {
                    Some(self.indexed(
                        base.ok_or("nonzero array has no storage")?,
                        index,
                        stride,
                        leaf.align,
                    )?)
                }
            }
            ProjectionKind::ConstantIndex(index) => {
                self.pattern_offset(base, &layout, &leaf, index, None)?
            }
            ProjectionKind::Subarray { start, end } => {
                self.pattern_offset(base, &layout, &leaf, start, Some(end))?
            }
        };
        self.projections[id.0 as usize] = Projected::Ready(address);
        Ok(())
    }
    pub(super) fn selected_field_offset(
        &self,
        layout: &Layout,
        field: Selection,
    ) -> Result<u64, &'static str> {
        let variant_ordinal = match field {
            Selection::Tuple(_) => 0,
            Selection::Nominal(id) => {
                self.context
                    .members
                    .variant(id)
                    .and_then(|v| self.context.members.variants.get(v.0 as usize))
                    .ok_or("missing selected field variant")?
                    .ordinal
            }
        };
        let ordinal = match field {
            Selection::Tuple(index) => index,
            Selection::Nominal(id) => self
                .context
                .members
                .ordinal(id)
                .ok_or("missing selected field ordinal")?,
        };
        self.field_offset(layout, variant_ordinal, ordinal as usize)
    }
    fn pattern_offset(
        &mut self,
        base: Option<Address>,
        layout: &Layout,
        leaf: &Layout,
        start: u64,
        end: Option<u64>,
    ) -> Result<Option<Address>, &'static str> {
        let offset = Self::pattern_byte_offset(layout, leaf, start, end)?;
        if leaf.size == 0 {
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
            Shape::Scalar => Err("aggregate initializer has scalar layout"),
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
