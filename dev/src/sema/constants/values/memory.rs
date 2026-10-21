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
    #[must_use]
    pub const fn pointer(location: LocationId) -> Self {
        Self(Value::Reference(Pointer::new(location.0)))
    }
}
impl Evaluator<'_> {
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
