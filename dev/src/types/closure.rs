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
        let declarations = self.closures.get_or_insert_with(Vec::new);
        let id = u32::try_from(declarations.len()).map_err(|_| "too many closure identities")?;
        declarations.push(kind);
        Ok(ClosureId(id))
    }

    #[must_use]
    pub fn closure_kind(&self, id: ClosureId) -> Option<CallKind> {
        self.closures.as_ref()?.get(id.0 as usize).copied()
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
