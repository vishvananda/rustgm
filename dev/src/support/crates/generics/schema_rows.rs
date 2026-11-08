//! Raw source contracts retain complete requests; only validation can publish data.
#[path = "advisory_contracts.rs"]
mod attributes;
#[path = "availability_columns.rs"]
mod availability;
#[path = "operation_rows.rs"]
mod bodies;
#[path = "bodymap.rs"]
pub(crate) mod bodymap;
#[path = "compiler_rows.rs"]
mod compiler_rows;
#[path = "profile_descriptor.rs"]
mod descriptor;
mod operation_contracts;
mod results_contract;
pub use compiler_rows::*;
#[path = "traitless_impls.rs"]
mod inherent_contracts;
#[path = "relation_inputs.rs"]
mod inputs;
#[path = "record_contracts.rs"]
mod nominal_contracts;
#[path = "sum_contracts.rs"]
mod sum_contracts;
pub use sum_contracts::{
    Constructor, ConstructorPayload, EnumColumns, EnumHeader, VariantField, VariantHeader,
    VariantShape, nominal_kind, positional,
};
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
    AvailabilityRows, AvailabilitySince, BindingAvailability, ConstBodyContracts, ConstBodyGrant,
    DeclarationAvailability,
};
pub use bodies::*;
pub(crate) use descriptor::canonical_order;
pub(crate) use descriptor::canonical_order_paid;
pub use descriptor::{
    BindingSource, Columns, CompilerOperationView, Descriptor, ImplementationDomain,
    ModuleBindingRow, ModuleContractRows, ModuleContractView, ModuleNamespace, ModuleRow,
    ModuleVisibility, Profile, TraitDomainRow, ValidatedRows, ValidatedView,
};
pub(crate) use descriptor::{
    ExportBinding, HeaderModifierColumns, HeaderModifierPolicy, HeaderModifierRecord,
    NamespaceColumns, ProvidedContractRows, UnpublishedNamespace, callable_path,
};
pub use inputs::*;
pub use scopes::{Definition, DefinitionKind, ScopeIndex};
pub(crate) use validation::HeaderRows;
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
    /// Version thirteen: a type slot's declared default. Validation admits only
    /// its owning trait's Self; consumers install it into imported generics.
    pub default: Option<TypeRef>,
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
    Nominal {
        declaration: DeclarationAddress,
        arguments: Range,
    },
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
    Projection(ProjectionRef),
    OpaqueFuture(OpaqueFutureRef),
    /// Version thirteen: a non-unit tuple whose element types are `roots`
    /// entries, in order. Unit keeps its scalar row.
    Tuple(Range),
    /// Version thirteen: the unsized `str` primitive, outside every scalar
    /// profile, reachable only behind a reference or pointer recipe.
    Str,
    /// Version thirteen: the never type `!`.
    Never,
    /// Version thirteen: `*const T` or `*mut T`. The pointee may be unsized;
    /// each consumer derives its sizedness from its own facts.
    RawPointer {
        mutable: bool,
        pointee: TypeRef,
    },
    /// Version thirteen: the unsized slice `[T]`, reachable only behind a
    /// reference or pointer recipe. Each consumer proves its element sized.
    Slice(TypeRef),
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Requirement {
    /// Canonical spelling only; each importer owns its actual feature identity.
    UnstableFeature(Box<str>),
    Trait {
        application: TraitApplicationRef,
        effect: Constness,
    },
    TypeOutlives {
        subject: TypeRef,
        shorter: RegionRef,
    },
    Outlives(RegionBound),
    /// Version thirteen: a declared-result binding such as the `-> U` of
    /// `F: FnOnce(T) -> U`. The projection recipe equals `value`; consumers
    /// normalize both again and never trust a provider normalization.
    ProjectionEqual {
        projection: TypeRef,
        value: TypeRef,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
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
    CompilerOperation(CompilerOperationRef),
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
/// Exact native definition identity; neither visibility nor body permission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactCallableExport {
    pub declaration: DeclarationRef,
    pub body: BodyRef,
    pub symbol: Box<str>,
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
pub struct NominalHeader {
    pub contract: HeaderContract,
    pub declaration: DeclarationRef,
    pub scope: DeclarationRef,
    pub self_type: TypeRef,
    pub fields: Range,
    pub inputs: Range,
    pub declared_inputs: u32,
    pub repr: crate::support::crates::RecordRepr,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NominalField {
    pub owner: DeclarationRef,
    pub ordinal: u32,
    pub name: String,
    pub ty: TypeRef,
    pub visibility: ModuleVisibility,
    pub site: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rows {
    pub associated_types: Vec<AssociatedTypeHeader>,
    /// Version thirteen's associated definitions and enum variant tables,
    /// allocated only when a packet has one; see [`ColdColumns`].
    pub cold: Option<Box<ColdColumns>>,
    pub projections: Vec<ProjectionRecipe>,
    pub opaque_futures: Vec<OpaqueFuture>,
    pub compiler_operations: Vec<CompilerOperation>,
    pub nominals: Vec<NominalHeader>,
    pub fields: Vec<NominalField>,
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
    pub exact_callables: Vec<ExactCallableExport>,
}
pub type SourceRows = Rows;
/// Cold column groups held out of line so packets without them, and every
/// hot record, pay no inline storage: version thirteen's associated-result
/// definitions and enum variant tables, plus the validated sparse original
/// index of a packet with compiler operations (absent from raw rows).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColdColumns {
    pub associated_definitions: Vec<AssociatedDefinition>,
    pub enums: EnumColumns,
    pub(crate) bodies: Option<std::sync::Arc<bodymap::Lookup>>,
}
static EMPTY_COLD: ColdColumns = ColdColumns {
    associated_definitions: Vec::new(),
    enums: EnumColumns {
        headers: Vec::new(),
        variants: Vec::new(),
        fields: Vec::new(),
    },
    bodies: None,
};
impl ColdColumns {
    pub(crate) fn of(cold: Option<&Self>) -> &Self {
        cold.unwrap_or(&EMPTY_COLD)
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.associated_definitions.is_empty() && self.enums.is_empty() && self.bodies.is_none()
    }
}
impl Rows {
    #[must_use]
    pub fn cold(&self) -> &ColdColumns {
        ColdColumns::of(self.cold.as_deref())
    }
    #[must_use]
    pub fn enums(&self) -> &EnumColumns {
        &self.cold().enums
    }
    #[must_use]
    pub fn associated_definitions(&self) -> &[AssociatedDefinition] {
        &self.cold().associated_definitions
    }
    /// The cold groups, allocated on first write. A writer accounts for
    /// `size_of::<ColdColumns>()` before the first call.
    pub fn cold_mut(&mut self) -> &mut ColdColumns {
        self.cold.get_or_insert_with(Box::default)
    }
}
