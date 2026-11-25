//! Declared opaque contracts and compiler operations grant no executable body.
use super::{DeclarationAddress, Range, TypeRef};
use crate::support::crates::owner_catalog::{BodyRef, DeclarationRef};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ProjectionRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpaqueFutureRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OpaqueResultRef(pub u32);
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

/// A trait implementation's definition of one declared associated result.
/// The value uses its impl frame or its own family frame with impl captures.
/// A fresh consumer proves the result bounds in that scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssociatedDefinition {
    pub declaration: DeclarationRef,
    pub value: TypeRef,
}

/// The sparse input range of one provider-owned associated family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssociatedDefinitionInputs {
    pub index: u32,
    pub inputs: Range,
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

/// One application of a function's return-position `impl Trait` opaque type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpaqueApplication {
    pub declaration: DeclarationRef,
    pub ordinal: u32,
    pub captures: Range,
}

/// Schema flags of one v14 return-position `impl Trait` opaque declaration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct OpaqueFlags(u8);
impl OpaqueFlags {
    pub const SIZED: u8 = 1;
    pub const COPY: u8 = 2;
    pub const CLONE: u8 = 4;
    pub const TRAIT_RESULT: u8 = 8;

    #[must_use]
    pub const fn new(sized: bool, trait_result: bool) -> Self {
        let mut bits = 0;
        if sized {
            bits |= Self::SIZED;
        }
        if trait_result {
            bits |= Self::TRAIT_RESULT;
        }
        Self(bits)
    }
    #[must_use]
    pub const fn with_copies(self, copy: bool, clone_: bool) -> Self {
        let mut bits = self.0;
        if copy {
            bits |= Self::COPY;
        }
        if clone_ {
            bits |= Self::CLONE;
        }
        Self(bits)
    }
    /// # Errors
    /// Rejects unknown flag bits outside the four schema booleans.
    pub const fn from_bits(bits: u8) -> Result<Self, &'static str> {
        if bits & !15 != 0 {
            return Err("unknown source opaque declaration flag");
        }
        Ok(Self(bits))
    }
    #[must_use]
    pub const fn bits(self) -> u8 {
        self.0
    }
    #[must_use]
    pub const fn sized(self) -> bool {
        self.0 & Self::SIZED != 0
    }
    #[must_use]
    pub const fn copy(self) -> bool {
        self.0 & Self::COPY != 0
    }
    #[must_use]
    pub const fn clone_(self) -> bool {
        self.0 & Self::CLONE != 0
    }
    #[must_use]
    pub const fn trait_result(self) -> bool {
        self.0 & Self::TRAIT_RESULT != 0
    }
}

/// One v14 return-position `impl Trait` opaque declaration, owned by its
/// defining function in that function's original signature frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OpaqueDeclaration {
    pub declaration: DeclarationRef,
    pub ordinal: u32,
    pub application: TypeRef,
    pub bounds: Range,
    pub hidden: Option<TypeRef>,
    pub flags: OpaqueFlags,
}

/// Out-of-line v14 opaque result applications (in type-row order) and
/// canonical declarations (in `(declaration, ordinal)` order).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OpaqueColumns {
    pub applications: Vec<OpaqueApplication>,
    pub declarations: Vec<OpaqueDeclaration>,
}
impl OpaqueColumns {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.applications.is_empty() && self.declarations.is_empty()
    }
}
