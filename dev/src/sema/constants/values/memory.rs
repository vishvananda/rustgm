//! Symbolic location construction shared with checked CFG constant execution.
use super::super::{
    Value,
    references::{Access, Place, Pointer},
};
use super::{Constant, Evaluator, Failure, LocationId};
use crate::{resolve::model::DefId, types::TypeId};

impl Constant {
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
            Some(Kind::Slice(element)) => (
                element,
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
            Some(Kind::Slice(selected))
                if super::Reader::new(facts).same_storage_type(selected, element) =>
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
