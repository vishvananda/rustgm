//! Original input and allocation requests remain separate from operations and proof.
use super::{DeclarationAddress, RegionBound, RegionRef, Requirement, SlotRef};
use crate::support::crates::{
    generics::{BinderRef, Range, TypeRef},
    owner_catalog::{BodyRef, DeclarationRef},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputSite {
    Body(u32),
    Declaration { owner: DeclarationRef, site: u32 },
}

/// Describes an exact existing construction; none is a satisfied caller premise.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnvironmentOrigin {
    Absent,
    Declaration(DeclarationRef),
    /// Local defining Self premise, not a declaration or executable const flag.
    ConstTraitDefault {
        member: DeclarationRef,
        trait_: DeclarationAddress,
    },
    /// Rebuild using the actual original Contract, never guessed type shape.
    Contract,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceConversion {
    pub dereferences: u32,
    pub unsize: bool,
    pub two_phase: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputKind {
    Variable {
        slot: u32,
        needs_provenance: bool,
    },
    Membership {
        region: RegionRef,
        /// Range in SourceRows.arguments; every element must be a region.
        captures: Range,
    },
    LocalBorrow(RegionRef),
    Equality(RegionBound),
    Outlives(RegionBound),
    Coercion {
        actual: TypeRef,
        expected: TypeRef,
        unsize: bool,
        reference: Option<ReferenceConversion>,
    },
    WellFormed(TypeRef),
    ScopedWellFormed {
        ty: TypeRef,
        frame: BinderRef,
    },
    TypeOutlives {
        ty: TypeRef,
        shorter: RegionRef,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Obligation {
    pub site: InputSite,
    pub kind: InputKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeOutlives {
    pub ty: TypeRef,
    pub shorter: RegionRef,
}

/// A root input may carry nested lexical scopes and scoped relation blocks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizationInput {
    pub site: InputSite,
    pub sources: Vec<InputSite>,
    /// Exact disjoint ranges in this body's explicit owned-allocation table.
    pub owned: Vec<Range>,
    pub equalities: Vec<RegionBound>,
    pub outlives: Vec<RegionBound>,
    pub types: Vec<TypeOutlives>,
    pub scoped: Option<Box<ScopedNormalization>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NormalizationScope {
    /// 0 is the root proof scope; 1..=index names a prior scope.
    pub parent: u32,
    pub frame: BinderRef,
    pub callable: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NormalizationBlock {
    /// 0 is the root proof scope; `1..=scopes.len()` names `scopes[scope - 1]`.
    pub scope: u32,
    pub universal_aliases: bool,
    pub origin: u32,
    /// Contiguous slot range in `original.variables`, or empty `0..0`.
    pub regions: Range,
    pub equalities: Vec<RegionBound>,
    pub outlives: Vec<RegionBound>,
    pub types: Vec<TypeOutlives>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedNormalization {
    pub scopes: Vec<NormalizationScope>,
    pub blocks: Vec<NormalizationBlock>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyQueries {
    /// None is active source policy with genuinely missing Copy, never fallback.
    pub declaration: Option<DeclarationAddress>,
    pub goals: Vec<Requirement>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocatedRequirement {
    pub site: InputSite,
    pub binder: Option<BinderRef>,
    pub value: Requirement,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BuiltinInputs {
    pub environment: EnvironmentOrigin,
    pub predicates: Vec<LocatedRequirement>,
    /// These are queries; a failed Copy query may retain the ordinary move path.
    pub source_copy: Option<CopyQueries>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyInputs {
    pub body: BodyRef,
    pub frame: BinderRef,
    pub parameters: Vec<SlotRef>,
    pub environment: EnvironmentOrigin,
    pub assumptions: Vec<RegionBound>,
    pub type_assumptions: Vec<LocatedRequirement>,
    pub obligations: Vec<Obligation>,
    pub normalizations: Vec<NormalizationInput>,
    pub builtins: Option<BuiltinInputs>,
}
