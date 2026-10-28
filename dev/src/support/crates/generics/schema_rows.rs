//! Raw source contracts retain complete requests; only validation can publish data.
#[path = "advisory_contracts.rs"]
mod attributes;
#[path = "availability_columns.rs"]
mod availability;
#[path = "operation_rows.rs"]
mod bodies;
#[path = "profile_descriptor.rs"]
mod descriptor;
#[path = "relation_inputs.rs"]
mod inputs;
#[path = "slot_domains.rs"]
mod scopes;
#[path = "authored_domains.rs"]
mod validation;
use super::{BinderRef, Range, Signature, TypeRef};
use crate::support::{
    bindings::LangItem,
    constness::Constness,
    crates::{
        Scalar,
        owner_catalog::{BodyOwner, BodyRef, DeclarationRef},
    },
    edition::Edition,
};
pub use attributes::*;
pub(crate) use availability::{AvailabilityAncestry, AvailabilityColumns};
pub use availability::{
    AvailabilityKind, AvailabilityLevel, AvailabilityOrigin, AvailabilityPolicy, AvailabilityRef,
    AvailabilityRows, AvailabilitySince, BindingAvailability, DeclarationAvailability,
};
pub use bodies::*;
pub(crate) use descriptor::canonical_order;
pub use descriptor::{
    BindingSource, Columns, Descriptor, ImplementationDomain, ModuleBindingRow, ModuleContractRows,
    ModuleContractView, ModuleNamespace, ModuleRow, ModuleVisibility, Profile, TraitDomainRow,
    ValidatedRows, ValidatedView,
};
pub(crate) use descriptor::{
    ExportBinding, HeaderModifierColumns, HeaderModifierPolicy, HeaderModifierRecord,
    NamespaceColumns, ProvidedContractRows, UnpublishedNamespace, callable_path,
};
pub use inputs::*;
pub use scopes::{Definition, DefinitionKind, ScopeIndex};
pub use validation::{
    AllocationOrdinals, DomainIndex, DomainWork, HeaderIndex, HeaderRef, InputWork, NumberingUnits,
    OperationPort, OriginalInputPlan, RequiredInput, RequiredKind,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArtifactRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DeclarationAddress {
    /// Zero is this artifact; later rows address its exact dependency inventory.
    pub artifact: ArtifactRef,
    pub declaration: DeclarationRef,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RegionRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TraitApplicationRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct FunctionApplicationRef(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Policy {
    pub edition: Edition,
    pub provider: ArtifactRef,
    pub roles: Range,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub definition: Definition,
    pub ordinal: u32,
    pub name: Option<String>,
    pub public: bool,
    pub sites: u32,
    /// Only genuine body-local declarations have this additional association.
    pub local: Option<LocalDeclaration>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalDeclaration {
    pub body: BodyRef,
    pub scope: u32,
    pub ordinal: u32,
}
/// Missing resolver definitions remain absent, including implicit lifetime slots.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterSlot {
    pub owner: DeclarationRef,
    pub ordinal: u32,
    /// Canonical type/region index, independently of mixed declaration order.
    pub index: u32,
    pub definition: Option<DeclarationRef>,
    pub kind: ParameterKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterKind {
    /// Some retains the actual sizing declaration; it grants no call permission.
    Type {
        sized: Option<DeclarationAddress>,
    },
    Region,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterScope {
    pub owner: DeclarationRef,
    pub parent: Option<DeclarationRef>,
    pub declared: Range,
    pub captures: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinderKind {
    /// Actual trait/impl declaration frame, including implicit captured regions.
    DeclarationHeader,
    OriginalFunction,
    PublicFunction,
    Clause,
    CallableType,
    CallSignature {
        body: BodyRef,
        value: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binder {
    pub owner: DeclarationRef,
    pub parent: Option<BinderRef>,
    pub kind: BinderKind,
    pub count: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Region {
    Static,
    Bound {
        binder: BinderRef,
        depth: u16,
        slot: u32,
    },
    Parameter {
        slot: SlotRef,
        depth: u16,
    },
    Variable {
        body: BodyRef,
        slot: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Argument {
    Type(TypeRef),
    Region(RegionRef),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Scalar(Scalar),
    Parameter(SlotRef),
    Reference {
        mutable: bool,
        region: RegionRef,
        referent: TypeRef,
    },
    Function(Signature),
    FunctionItem(FunctionApplicationRef),
    ApplicationSignature(FunctionApplicationRef),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraitApplication {
    pub declaration: DeclarationAddress,
    pub subject: TypeRef,
    pub arguments: Range,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionApplication {
    pub declaration: DeclarationAddress,
    pub arguments: Range,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Requirement {
    Trait {
        application: TraitApplicationRef,
        effect: Constness,
    },
    TypeOutlives {
        subject: TypeRef,
        shorter: RegionRef,
    },
    Outlives(RegionBound),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Clause {
    pub owner: DeclarationRef,
    pub site: u32,
    pub binder: Option<BinderRef>,
    pub requirement: Requirement,
    /// Actual canonical Self subject, including typed generated sizing defaults.
    /// Only the exact trait-contract prefix may supply this classification;
    /// canonical type equality cannot invent Self syntax or an own-trait premise.
    pub self_subject: Option<SlotRef>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionBound {
    pub longer: RegionRef,
    pub shorter: RegionRef,
}
/// Checked structural claims, not implementation, cycle or const-drop permission.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TraitFlags(u16);
impl TraitFlags {
    pub const UNSAFE: u16 = 1;
    pub const CONST: u16 = 2;
    pub const AUTO: u16 = 4;
    pub const DENY_IMPL: u16 = 8;
    pub const FUNDAMENTAL: u16 = 16;
    pub const SPECIALIZATION: u16 = 32;
    pub const COINDUCTIVE: u16 = 64;
    pub const DYN_INCOMPATIBLE: u16 = 128;
    /// Structural claim requiring the complete version-two domain row.
    pub const RESTRICTED_IMPL: u16 = 256;
    /// # Errors
    /// Rejects claims outside this exact supported structural vocabulary.
    pub fn from_bits(bits: u16) -> Result<Self, &'static str> {
        if bits & !511 != 0 {
            return Err("unknown source trait attribute claim");
        }
        Ok(Self(bits))
    }
    #[must_use]
    pub const fn bits(self) -> u16 {
        self.0
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraitHeader {
    pub declaration: DeclarationRef,
    pub scope: DeclarationRef,
    pub self_definition: DeclarationRef,
    pub self_type: TypeRef,
    pub flags: TraitFlags,
    pub region_captures: u32,
    pub region_bounds: Range,
    pub inputs: Range,
    /// Exact original trait-contract prefix, before implicit own-trait premises
    /// and the later elaborated declaration assumption environment.
    pub canonical_inputs: u32,
    pub declared_inputs: u32,
    pub written_parameters: u32,
    /// Literal colon-bound indices, independently of complete semantic Self
    /// requirements (including defaults and written where-Self clauses).
    pub written_supertraits: Range,
    pub members: Range,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImplHeader {
    pub self_definition: DeclarationRef,
    pub scope: DeclarationRef,
    pub self_type: TypeRef,
    pub trait_application: Option<TraitApplicationRef>,
    pub unsafe_: bool,
    pub constant: bool,
    pub negative: bool,
    pub region_captures: u32,
    pub region_bounds: Range,
    pub inputs: Range,
    pub declared_inputs: u32,
    pub members: Range,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MemberLink {
    /// None only for an actual inherent member, never a trait implementation.
    pub trait_member: Option<DeclarationAddress>,
    /// None denotes its actual default, not omitted conformance checking.
    pub implementation: Option<DeclarationRef>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionBody {
    Absent,
    Original(BodyRef),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Formal {
    pub declaration: Option<DeclarationRef>,
    pub ordinal: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Receiver {
    pub formal: u32,
    pub base: TypeRef,
    pub references: u32,
    pub mutable_binding: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionScheme {
    pub total: u32,
    pub captures: u32,
    pub early: Range,
    pub late: Range,
    pub bounds: Range,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionHeader {
    pub declaration: DeclarationRef,
    pub scope: DeclarationRef,
    pub parent: Option<DeclarationRef>,
    pub callable: Signature,
    pub original: Signature,
    pub regions: RegionScheme,
    pub inputs: Range,
    pub declared_inputs: u32,
    pub formals: Range,
    pub receiver: Option<Receiver>,
    /// Declaration eligibility data; ordinary proof/payment remains mandatory.
    pub constness: Constness,
    pub body: FunctionBody,
}
/// Complete numbering-unit data. Neither allocation order nor an optional
/// universal alias ledger is a region proof or defining-environment permission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BodyUnit {
    pub body: BodyRef,
    pub universals: u32,
    /// Owned-variable slot ordinals in the actual source unit allocation order.
    pub allocation_order: Vec<u32>,
    /// Exact owned-variable slot for every declaration universal, when present.
    pub aliases: Option<Vec<u32>>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RoleBinding {
    pub role: LangItem,
    pub declaration: DeclarationAddress,
}
/// Raw, unpublished tables. No artifact decoder/encoder accepts these yet.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rows {
    pub policy: Policy,
    pub declarations: Vec<Declaration>,
    pub body_owners: Vec<BodyOwner>,
    pub slots: Vec<ParameterSlot>,
    pub scopes: Vec<ParameterScope>,
    pub binders: Vec<Binder>,
    pub types: Vec<Type>,
    pub roots: Vec<TypeRef>,
    pub regions: Vec<Region>,
    pub arguments: Vec<Argument>,
    pub clauses: Vec<Clause>,
    pub clause_indices: Vec<u32>,
    pub region_bounds: Vec<RegionBound>,
    pub region_slots: Vec<SlotRef>,
    pub trait_applications: Vec<TraitApplication>,
    pub function_applications: Vec<FunctionApplication>,
    pub traits: Vec<TraitHeader>,
    pub implementations: Vec<ImplHeader>,
    pub functions: Vec<FunctionHeader>,
    pub formals: Vec<Formal>,
    pub members: Vec<DeclarationRef>,
    pub member_links: Vec<MemberLink>,
    pub roles: Vec<RoleBinding>,
    pub originals: Vec<Original>,
    pub body_inputs: Vec<BodyInputs>,
    pub body_units: Vec<BodyUnit>,
}
pub type SourceRows = Rows;
