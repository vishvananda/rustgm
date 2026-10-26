//! Symbolic location construction shared with checked CFG constant execution.
use super::super::{
    Value,
    references::{Access, Place, Pointer},
};
use super::{Constant, Evaluator, Failure, LocationId, Metadata, Reader};
use crate::{resolve::model::DefId, types::TypeId};

impl Constant {
    /// A numeric raw address supplies no allocation, reference or dereference authority.
    #[must_use]
    pub(crate) const fn raw_address(bits: u64) -> Self {
        Self(Value::Address {
            bits,
            metadata: None,
        })
    }

    #[must_use]
    pub const fn location(self) -> Option<LocationId> {
        if let Value::Reference(pointer) = self.0 {
            Some(LocationId(pointer.place))
        } else {
            None
        }
    }
    #[must_use]
    pub const fn write_permission(self) -> Option<super::WritePermission> {
        if let Value::Reference(pointer) = self.0 {
            Some(pointer.write)
        } else {
            None
        }
    }
    /// Restriction never changes the backing allocation or restores write authority.
    #[must_use]
    pub const fn restrict_shared(mut self) -> Self {
        if let Value::Reference(pointer) = &mut self.0 {
            pointer.write = super::WritePermission::SharedDerived;
        }
        self
    }
    /// Follow only this pointer's retained ordinary projection ancestry.
    /// The incoming write restriction belongs to the value, not the allocation.
    pub(crate) fn projection_parent(
        self,
        reader: super::Reader<'_>,
    ) -> Result<(Self, u64), &'static str> {
        let Value::Reference(mut pointer) = self.0 else {
            return Err("constant projection parent requires a pointer value");
        };
        let super::Origin::Projection { base, index } =
            reader.location(LocationId(pointer.place))?.origin
        else {
            return Err("constant pointer has no ordinary projection parent");
        };
        pointer.place = base.0;
        pointer.metadata = None;
        Ok((Self(Value::Reference(pointer)), index))
    }
    #[must_use]
    pub const fn pointer(location: LocationId) -> Self {
        Self(Value::Reference(Pointer::new(location.0)))
    }
}
impl Evaluator<'_> {
    /// Give an addressed literal its canonical immutable backing on first dereference.
    /// Other pointer values and explicitly selected metadata remain unchanged.
    /// # Errors
    /// Rejects missing literal bytes or exhausted type/backing storage.
    pub fn literal_address(&mut self, value: Constant) -> Result<Constant, Failure> {
        let (Value::Bytes(text) | Value::Text(text) | Value::BytePointer { text, .. }) = value.0
        else {
            return Ok(value);
        };
        let facts = &mut *self.queries.typing.facts;
        let length = facts.texts.get(text)?.len() as u64;
        let ty = facts.types.intern(crate::types::Kind::Array {
            element: TypeId::primitive(crate::types::Primitive::U8),
            length,
        })?;
        let address = Constant(facts.constant_references.literal_place(text, ty)?);
        let metadata = match value.0 {
            Value::Text(_) => Some(super::Metadata::Length(length)),
            _ => value.metadata(),
        };
        metadata.map_or(Ok(address), |metadata| address.with_metadata(metadata))
    }
    /// Execute an already checked reference adjustment without losing slice metadata.
    /// The typed CFG's legacy `Constrain` operation also carries direct array unsizing.
    /// This reads its representation change; it does not establish conversion eligibility.
    /// # Errors
    /// Rejects an array-to-slice conversion whose value has no address representation.
    pub fn constrain(
        &self,
        value: Constant,
        source: TypeId,
        target: TypeId,
    ) -> Result<Constant, Failure> {
        use crate::types::Kind;
        if source == target {
            return Ok(value);
        }
        let types = &self.facts().types;
        if let (
            Some(Kind::Reference {
                referent: source, ..
            }),
            Some(Kind::Reference { .. }),
        ) = (types.kind(source), types.kind(target))
        {
            return self.adjust_reference(value, source, target);
        }
        Ok(value)
    }
    /// Form the checked reference view of an existing place, preserving its identity.
    /// Direct reborrows can attach slice metadata without a separate CFG unsize node.
    /// # Errors
    /// Rejects a non-reference target or a metadata conversion without an address.
    pub fn adjust_reference(
        &self,
        mut value: Constant,
        source: TypeId,
        target: TypeId,
    ) -> Result<Constant, Failure> {
        use crate::types::Kind;
        let types = &self.facts().types;
        let Some(Kind::Reference {
            referent, mutable, ..
        }) = types.kind(target)
        else {
            return Err("constant reference adjustment lacks a reference target".into());
        };
        if !mutable {
            value = value.restrict_shared();
        }
        if let (Some(Kind::Array { length, .. }), Some(Kind::Slice(_))) =
            (types.kind(source), types.kind(referent))
        {
            return value.unsize(length);
        }
        Ok(value)
    }
    /// Reserve storage identity independently of initialization or reads.
    /// # Errors
    /// Diagnoses exhausted location identities.
    pub fn allocation(&mut self, value: Option<Constant>, ty: TypeId) -> Result<Constant, Failure> {
        self.queries
            .typing
            .facts
            .constant_references
            .insert(Place {
                value: value.map(|v| v.0),
                ty,
                access: Access::Owned,
                packed: None,
                projection: None,
            })
            .map(Constant)
            .map_err(Failure::Invalid)
    }
    /// Retain a field's storage identity without demanding its current contents.
    /// # Errors
    /// Rejects invalid base identities and exhausted location storage.
    pub fn projection(
        &mut self,
        base: Constant,
        index: u64,
        ty: TypeId,
    ) -> Result<Constant, Failure> {
        let Value::Reference(base) = base.0 else {
            return Err(Failure::Invalid(
                "constant projection base is not a pointer",
            ));
        };
        self.queries
            .typing
            .facts
            .constant_references
            .project(
                base,
                index,
                Place {
                    value: None,
                    ty,
                    access: Access::Owned,
                    packed: None,
                    projection: None,
                },
            )
            .map(Constant)
            .map_err(Failure::Invalid)
    }
    /// Form a static address without evaluating the initializer.
    /// # Errors
    /// Rejects unknown types and exhausted storage identities.
    pub fn static_address(&mut self, definition: DefId, ty: TypeId) -> Result<Constant, Failure> {
        self.queries
            .typing
            .facts
            .constant_references
            .static_place(definition, ty)
            .map(Constant)
            .map_err(Failure::Invalid)
    }
    /// Publish only the complete selected instance after checking its canonical result.
    pub(crate) fn complete_associated(
        &mut self,
        instance: super::Instance,
        result: TypeId,
        value: Constant,
    ) -> Result<(), Failure> {
        instance
            .publish(self.queries.typing.facts, Some(result), value)
            .map_err(Failure::Invalid)
    }
    /// Publish one checked demanded initializer, preserving its declaration identity.
    pub fn complete_definition(&mut self, definition: DefId, value: Constant) {
        self.queries
            .typing
            .facts
            .constant_values
            .insert(definition, value.0);
    }
}

impl Reader<'_> {
    /// Retain the immediate physical field under an already checked unsized view.
    /// Logical bounds and conversion proofs remain separate from this storage identity.
    pub(crate) fn tail_storage(
        self,
        owner: TypeId,
        actual: TypeId,
        index: u64,
        target: TypeId,
        metadata: Metadata,
    ) -> Result<TypeId, &'static str> {
        use crate::types::Kind;
        let types = &self.facts.types;
        let same_owner = match (types.kind(owner), types.kind(actual)) {
            (Some(Kind::Adt(left, _)), Some(Kind::Adt(right, _))) => left == right,
            (Some(Kind::Tuple(left)), Some(Kind::Tuple(right))) => {
                types.list(left).ok_or("missing logical tail fields")?.len()
                    == types
                        .list(right)
                        .ok_or("missing physical tail fields")?
                        .len()
            }
            _ => false,
        };
        if !same_owner {
            return Err("constant tail storage has a different physical owner");
        }
        let stored = self.stored_child(actual, None, index)?;
        if stored == target {
            return Ok(stored);
        }
        let compatible = match (types.kind(target), types.kind(stored), metadata) {
            (
                Some(Kind::Slice(selected)),
                Some(Kind::Array { element, length }),
                Metadata::Length(n),
            ) => selected == element && n <= length,
            (Some(Kind::Adt(left, _)), Some(Kind::Adt(right, _)), _) => left == right,
            (Some(Kind::Tuple(left)), Some(Kind::Tuple(right)), _) => {
                types.list(left).ok_or("missing logical tail fields")?.len()
                    == types
                        .list(right)
                        .ok_or("missing physical tail fields")?
                        .len()
            }
            (
                Some(Kind::Dynamic { .. }),
                _,
                Metadata::Object {
                    concrete,
                    target: object,
                },
            ) => concrete == stored && object == target,
            _ => false,
        };
        if !compatible {
            return Err("constant tail storage differs from its selected metadata or type");
        }
        Ok(stored)
    }
    /// A completed DST's final unsized field inherits its existing metadata.
    /// This selects a physical field; it establishes no source conversion proof.
    pub(crate) fn projected_metadata(
        self,
        owner: TypeId,
        field: crate::sema::members::Selection,
        target: TypeId,
        metadata: Option<Metadata>,
    ) -> Result<Option<Metadata>, &'static str> {
        use crate::types::Kind;
        let types = &self.facts.types;
        if types
            .is_sized(owner)
            .ok_or("missing projected owner sizedness")?
        {
            return if metadata.is_none() {
                Ok(None)
            } else {
                Err("sized projected owner carries unsized metadata")
            };
        }
        if types.contains_parameters(owner) != Some(false)
            || types.contains_projections(owner) != Some(false)
            || types.contains_opaques(owner) != Some(false)
        {
            return Err("constant tail projection requires completed concrete storage");
        }
        let (index, last) = match (types.kind(owner), field) {
            (Some(Kind::Tuple(fields)), crate::sema::members::Selection::Tuple(index)) => {
                let fields = types.list(fields).ok_or("missing projected tuple fields")?;
                let index = usize::try_from(index).map_err(|_| "tuple field exceeds host size")?;
                if index >= fields.len() {
                    return Err("projected tuple field exceeds its owner");
                }
                (index as u64, index + 1 == fields.len())
            }
            (Some(Kind::Adt(item, _)), crate::sema::members::Selection::Nominal(field)) => {
                let definition = self
                    .facts
                    .aggregates
                    .get(item)
                    .ok_or("missing completed projected nominal")?;
                if definition.kind != crate::types::aggregate::Kind::Struct {
                    return Err("unsized field projection requires a struct");
                }
                let members = &self.facts.members;
                let variant = members
                    .variant(field)
                    .ok_or("missing projected field variant")?;
                let group = members
                    .variants
                    .get(variant.0 as usize)
                    .ok_or("missing projected field group")?;
                let index = members
                    .ordinal(field)
                    .ok_or("missing projected field ordinal")?;
                if group.owner != item
                    || group.ordinal != 0
                    || members.variants(item) != [variant]
                    || group.start.checked_add(index) != Some(field.0)
                    || field.0 >= group.end
                {
                    return Err("projected tail field has a different owner or ordinal");
                }
                (u64::from(index), field.0.checked_add(1) == Some(group.end))
            }
            _ => return Err("unsized field projection differs from its checked owner"),
        };
        if self.stored_child(owner, None, index)? != target {
            return Err("projected tail field differs from its checked type");
        }
        match types.is_sized(target) {
            Some(true) => Ok(None),
            Some(false) if last => metadata
                .map(Some)
                .ok_or("constant unsized tail lacks metadata"),
            Some(false) => Err("unsized projected field is not its owner's final field"),
            None => Err("missing projected field sizedness"),
        }
    }
}

impl Evaluator<'_> {
    /// Select a checked sequence's storage without reading or copying its elements.
    /// # Errors
    /// Rejects inconsistent element types, missing metadata and out-of-bounds ranges.
    pub fn subsequence(
        &mut self,
        base: Constant,
        from: u64,
        length: u64,
        ty: TypeId,
    ) -> Result<Constant, Failure> {
        use crate::types::Kind;
        let Value::Reference(pointer) = base.0 else {
            return Err("constant range requires symbolic storage".into());
        };
        let facts = &mut *self.queries.typing.facts;
        let original = facts.constant_references.get(base.0)?;
        let (element, bound) = match facts.types.kind(original.ty) {
            Some(Kind::Array { element, length }) => {
                (element, base.length().unwrap_or(length).min(length))
            }
            Some(Kind::Slice(_) | Kind::Primitive(crate::types::Primitive::Str)) => (
                Reader::new(facts).sequence_element(original.ty)?,
                base.length()
                    .ok_or("constant slice lacks length metadata")?,
            ),
            _ => return Err("constant range requires array or slice storage".into()),
        };
        let slice = match facts.types.kind(ty) {
            Some(Kind::Array {
                element: selected,
                length: n,
            }) if super::Reader::new(facts).same_storage_type(selected, element) && n == length => {
                false
            }
            Some(Kind::Slice(_) | Kind::Primitive(crate::types::Primitive::Str))
                if Reader::new(facts)
                    .same_storage_type(Reader::new(facts).sequence_element(ty)?, element) =>
            {
                true
            }
            _ => {
                return Err(
                    "constant range differs from its checked element type or length".into(),
                );
            }
        };
        if from.checked_add(length).is_none_or(|end| end > bound) {
            return Err(Failure::Fault("constant range is out of bounds"));
        }
        let value = Constant(facts.constant_references.subsequence(
            pointer,
            from,
            length,
            Place {
                value: None,
                ty,
                access: original.access,
                packed: original.packed,
                projection: None,
            },
        )?);
        if slice {
            value.unsize(length)
        } else {
            Ok(value)
        }
    }
    /// Snapshot a completed sequence with a shared immutable range recipe.
    /// # Errors
    /// Rejects malformed sequence storage, range bounds and exhausted recipe storage.
    pub fn sequence_range(
        &mut self,
        value: Constant,
        from: u64,
        length: u64,
    ) -> Result<Constant, Failure> {
        let facts = &mut *self.queries.typing.facts;
        facts
            .constant_products
            .range(value.0, from, length, &facts.texts)
            .map(Constant)
            .map_err(Failure::Invalid)
    }
}
