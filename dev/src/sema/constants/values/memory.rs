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
    /// Publish one checked demanded initializer, preserving its declaration identity.
    pub fn complete_definition(&mut self, definition: DefId, value: Constant) {
        self.queries
            .typing
            .facts
            .constant_values
            .insert(definition, value.0);
    }
}
