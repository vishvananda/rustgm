//! Version-fourteen source closure headers, types and nested output links.
use super::{BinderRef, Range, RegionRef, TypeRef};
use crate::support::{
    call_traits::CallTrait,
    crates::owner_catalog::{BodyRef, DeclarationRef},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClosureRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClosureTypeRef(pub u32);

/// One applied closure type row, dense in type-row order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosureType {
    pub closure: ClosureRef,
    pub arguments: Range,
    pub signature: TypeRef,
    pub environment: TypeRef,
}

/// Captured closure execution contract; public inputs remain on `ClosureHeader`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosureExecution {
    pub self_ty: TypeRef,
    pub signature: TypeRef,
    pub universals: u32,
    pub receiver: CallTrait,
}

/// One nested closure output-interface bridge mapping a child slot to its parent region.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosureBridge {
    pub slot: u16,
    pub parent: RegionRef,
    pub output: bool,
}

/// One capture formation root with both child and parent lifetime views.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosureFormationInput {
    pub root: u32,
    pub child: TypeRef,
    pub parent: TypeRef,
}

/// Captured closure formation data exported to its enclosing parent body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosureFormation {
    pub inputs: Range,
    pub inherited: u32,
    pub callable_slots: Range,
}

/// Nested output interface data of a closure original.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosureOutput {
    pub revision: u32,
    pub universals: u32,
    pub bridges: Range,
    pub formation: Option<ClosureFormation>,
    pub creator_only_from: Option<u32>,
}

/// One transported closure body header, ordered by strictly increasing `body`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ClosureHeader {
    pub body: BodyRef,
    pub owner: DeclarationRef,
    pub binder: BinderRef,
    pub call_kind: CallTrait,
    pub constant: bool,
    pub signature: TypeRef,
    pub universals: u32,
    pub inherited: u32,
    pub public_binders: u32,
    pub execution: Option<ClosureExecution>,
    pub output: Option<ClosureOutput>,
    pub require_result_formation: bool,
}
impl ClosureHeader {
    /// Total original universals owned by this closure's body binder.
    #[must_use]
    pub fn contract_universals(&self) -> u32 {
        let base = self
            .execution
            .map_or(self.universals, |execution| execution.universals);
        self.output
            .map_or(base, |output| base.max(output.universals))
    }

    /// Checked body signature: execution signature when captured, public signature otherwise.
    #[must_use]
    pub const fn body_signature(&self) -> TypeRef {
        match self.execution {
            Some(execution) => execution.signature,
            None => self.signature,
        }
    }
}

/// Out-of-line version-fourteen closure tables held inside [`super::ColdColumns`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ClosureColumns {
    pub types: Vec<ClosureType>,
    pub headers: Vec<ClosureHeader>,
    pub bridges: Vec<ClosureBridge>,
    pub formation_inputs: Vec<ClosureFormationInput>,
    pub callable_slots: Vec<u16>,
}
impl ClosureColumns {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
            && self.headers.is_empty()
            && self.bridges.is_empty()
            && self.formation_inputs.is_empty()
            && self.callable_slots.is_empty()
    }

    #[must_use]
    pub fn header(&self, reference: ClosureRef) -> Option<&ClosureHeader> {
        self.headers.get(reference.0 as usize)
    }

    #[must_use]
    pub fn header_for_body(&self, body: BodyRef) -> Option<(ClosureRef, &ClosureHeader)> {
        let index = self
            .headers
            .binary_search_by_key(&body.0, |header| header.body.0)
            .ok()?;
        let reference = u32::try_from(index).ok()?;
        Some((ClosureRef(reference), &self.headers[index]))
    }
}
