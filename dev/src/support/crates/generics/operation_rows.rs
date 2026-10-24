//! Original-owned operation/region rows contain no serialized proof or source handles.
use super::{FunctionApplicationRef, RegionRef, TraitApplicationRef};
use crate::support::{
    constness::Constness,
    crates::{
        generics::{self as g, BinderRef, Range, TypeRef},
        owner_catalog::{BodyRef, DeclarationRef},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Local(u32),
    Projection(u32),
}
/// Exactly a retained reference Dereference projection. Raw dereferences, fields,
/// captures and other projections are not encodable by this first source form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projection {
    pub base: Place,
    pub ty: TypeRef,
    pub region: RegionRef,
    pub mutable: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedRegion {
    pub site: u32,
    pub allocation: RegionAllocation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionAllocation {
    Inference {
        needs_provenance: bool,
    },
    /// Ordinal of the exact owning raw normalization input in this body.
    Normalization(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    Unit,
    Read(Place),
    /// Immediate only. The producer must reject `TwoPhase` and `UniqueImmutable`;
    /// absence of a kind field must never erase their distinct semantics.
    Borrow(Place),
    Constrain(u32),
    Function(FunctionApplicationRef),
    Call {
        signature: TypeRef,
        callee: u32,
        arguments: Range,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Value {
    pub ty: TypeRef,
    pub kind: ValueKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    StorageLive(u32),
    StorageDead(u32),
    StorageEnd { first: u32, last: u32 },
    Project(u32),
    Evaluate(u32),
    Store { place: Place, value: u32 },
    CheckInitialized(Place),
    Mention(Place),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Statement {
    pub site: u32,
    pub action: Action,
}
/// Exact late opening; the full resulting signature must match, not its arity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Opening {
    pub binder: BinderRef,
    pub regions: Range,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallRequest {
    pub value: u32,
    pub site: u32,
    pub target: FunctionApplicationRef,
    pub method: Option<TraitApplicationRef>,
    pub signature: TypeRef,
    pub opening: Option<Opening>,
    /// Defining request data. Reached instantiation still derives and proves its
    /// actual runtime/const obligations; this is never a paid call witness.
    pub effect: Constness,
}
/// Exact owned-rvalue storage inputs; ordinary destruction proof remains required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Release {
    pub value: u32,
    pub location: g::Location,
    pub exits_before: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedValues {
    pub created: Vec<g::Temporary>,
    pub returned: Vec<Release>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallContext {
    Runtime,
    ConstDefinition,
    ClosedConst,
}
impl CallContext {
    #[must_use]
    pub const fn request(self) -> Constness {
        match self {
            Self::Runtime => Constness::Ordinary,
            Self::ConstDefinition => Constness::Conditional,
            Self::ClosedConst => Constness::Const,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Original {
    pub body: BodyRef,
    pub declaration: DeclarationRef,
    pub result: TypeRef,
    pub sites: u32,
    pub context: CallContext,
    pub variables: Vec<OwnedRegion>,
    pub locals: Vec<g::Local>,
    pub parameter_bindings: Vec<g::ParameterBinding>,
    pub projections: Vec<Projection>,
    pub values: Vec<Value>,
    pub statements: Vec<Statement>,
    pub blocks: Vec<g::Block>,
    pub scopes: Vec<g::Scope>,
    pub exits: Vec<g::Exit>,
    pub temporaries: Vec<g::Temporary>,
    pub owned: Option<OwnedValues>,
    pub arguments: Vec<u32>,
    pub openings: Vec<RegionRef>,
    pub calls: Vec<CallRequest>,
}
