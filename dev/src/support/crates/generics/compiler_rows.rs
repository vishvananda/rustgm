//! Declared opaque contracts and compiler operations grant no executable body.
use super::{DeclarationAddress, Range, TypeRef};
use crate::support::crates::owner_catalog::{BodyRef, DeclarationRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpaqueFutureRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompilerOperationRef(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssociatedTypeHeader {
    pub declaration: DeclarationRef,
    pub parent: DeclarationRef,
    pub scope: DeclarationRef,
    pub projection: TypeRef,
    pub inputs: Range,
    pub declared_inputs: u32,
    pub bounds: Range,
    pub sized: bool,
    pub copy: bool,
    pub clone_: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionRecipe {
    pub declaration: DeclarationAddress,
    pub subject: TypeRef,
    pub trait_application: super::TraitApplicationRef,
    pub arguments: Range,
}

/// Applications share identity by declaration/ordinal; captures are never erased.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpaqueFuture {
    pub declaration: DeclarationRef,
    pub ordinal: u32,
    pub captures: Range,
    pub output: TypeRef,
    pub future_trait: DeclarationAddress,
    pub output_member: DeclarationAddress,
    pub trait_result: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilerOperationKind {
    AsyncDropInPlace { target_formal: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompilerOperation {
    pub body: BodyRef,
    pub declaration: DeclarationRef,
    pub future: TypeRef,
    pub output: TypeRef,
    pub kind: CompilerOperationKind,
}
