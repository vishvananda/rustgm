//! Closure identity is nominal; callable contracts and stored captures are separate edges.
use super::{
    GenericArgs, Store, TypeId,
    properties::{Properties, Property},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClosureId(pub u32);

/// The strongest receiver permission required by the body, independent of `move` capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallKind {
    Shared,
    Mutable,
    Once,
}

/// A reserved source body cannot authorize a call before its capture uses finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Declaration {
    Pending,
    Complete(CallKind),
}
impl CallKind {
    #[must_use]
    pub const fn implements(self, required: Self) -> bool {
        self as u8 <= required as u8
    }
}

impl Store {
    /// Publish immutable body call permissions before interning its canonical type.
    /// Source identity and capture places remain owned by semantic analysis.
    /// # Errors
    /// Rejects compact identity overflow.
    pub fn declare_closure(&mut self, kind: CallKind) -> Result<ClosureId, &'static str> {
        let id = self.reserve_closure()?;
        self.complete_closure(id, kind)?;
        Ok(id)
    }

    /// Reserve identity while the body's signature and capture terms are inferred.
    /// Pending identities cannot be interned as canonical closure types.
    /// # Errors
    /// Rejects compact identity overflow.
    pub fn reserve_closure(&mut self) -> Result<ClosureId, &'static str> {
        let declarations = self.closures.get_or_insert_with(Vec::new);
        let id = u32::try_from(declarations.len()).map_err(|_| "too many closure identities")?;
        declarations.push(Declaration::Pending);
        Ok(ClosureId(id))
    }

    /// Seal one body's call permission before canonical type properties can exist.
    /// # Errors
    /// Rejects foreign identities and any attempt to revise a completed declaration.
    pub fn complete_closure(&mut self, id: ClosureId, kind: CallKind) -> Result<(), &'static str> {
        let declaration = self
            .closures
            .as_mut()
            .and_then(|declarations| declarations.get_mut(id.0 as usize))
            .ok_or("missing closure declaration")?;
        if *declaration != Declaration::Pending {
            return Err("closure call permission is already complete");
        }
        *declaration = Declaration::Complete(kind);
        Ok(())
    }

    #[must_use]
    pub fn closure_declaration(&self, id: ClosureId) -> Option<Declaration> {
        self.closures.as_ref()?.get(id.0 as usize).copied()
    }

    #[must_use]
    pub fn closure_kind(&self, id: ClosureId) -> Option<CallKind> {
        match self.closure_declaration(id)? {
            Declaration::Pending => None,
            Declaration::Complete(kind) => Some(kind),
        }
    }

    pub(super) fn closure_properties(
        &self,
        arguments: GenericArgs,
        signature: TypeId,
        environment: TypeId,
    ) -> Result<Properties, &'static str> {
        let mut result = *self
            .properties
            .get(environment.0 as usize)
            .ok_or("invalid closure environment")?;
        let signature = *self
            .properties
            .get(signature.0 as usize)
            .ok_or("invalid closure signature")?;
        result.include(signature);
        self.argument_properties(arguments, &mut result)?;
        // A closure always owns sized fields. Invalid captures cannot become a DST.
        result.set(
            Property::WellFormed,
            result.has(Property::WellFormed) && result.has(Property::Sized),
        );
        result.set(Property::Sized, true);
        Ok(result)
    }
}
