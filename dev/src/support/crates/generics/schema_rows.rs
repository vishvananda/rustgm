//! Raw source contracts retain complete requests; only validation can publish data.
#[path = "advisory_contracts.rs"]
mod attributes;
#[path = "availability_columns.rs"]
mod availability;
#[path = "operation_rows.rs"]
mod bodies;
#[path = "bodymap.rs"]
pub(crate) mod bodymap;
#[path = "bridged_closures.rs"]
mod bridged_closures;
#[path = "compiler_rows.rs"]
mod compiler_rows;
pub use bridged_closures::*;
#[path = "valued_defaults.rs"]
pub(crate) mod const_defaults;
#[path = "profile_descriptor.rs"]
mod descriptor;
mod interval_roles;
mod operation_contracts;
pub use const_defaults::{ConstDefault, ConstDefaultValue};
mod results_contract;
#[path = "global_columns.rs"]
mod statics;
pub use compiler_rows::*;
pub use statics::{StaticColumns, StaticHeader, StaticInitializer};
pub(crate) use statics::{StaticShape, StaticTypeTable, read_bits as read_static_bits};
#[path = "free_constants.rs"]
mod free_constants;
#[path = "traitless_impls.rs"]
mod inherent_contracts;
#[path = "relation_inputs.rs"]
mod inputs;
#[path = "record_contracts.rs"]
mod nominal_contracts;
#[path = "sum_contracts.rs"]
mod sum_contracts;
pub use free_constants::{ConstantBody, ConstantHeader, PromotedHandler};
pub(crate) use free_constants::{inline_scalar_width, inline_tuple_width};
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
    DeclarationAvailability, DeprecatedDeclaration, DeprecationColumns, DeprecationNotice,
    DeprecationSince,
};
pub(crate) use bodies::ScalarType;
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
    NamespaceColumns, ProvidedContractRows, UnpublishedNamespace, callable_path, nested_original,
};
pub use inputs::*;
pub(crate) use scopes::{ConstFormalType, const_formal_type, const_parameter_type, const_value};
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
    /// Version thirteen: a const slot of an integer, bool or char type. It
    /// shares the index space of type slots and has no sizing or default.
    Const {
        ty: TypeRef,
    },
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
    DynamicType,
    CallSignature {
        body: BodyRef,
        value: u32,
    },
    ClosureOriginal {
        body: BodyRef,
    },
    AnonymousOriginal {
        body: BodyRef,
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
    /// Version thirteen: a const argument, whose term is held out of line in
    /// [`ColdColumns::consts`] and written in place at this reference.
    Const(ConstRef),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConstRef(pub u32);
/// A transported const term: an evaluated value or a rigid const slot.
/// Generic argument terms use only scalar const parameter types.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstTerm {
    Value(ConstValue),
    Parameter(SlotRef),
    Variant(ConstVariant),
}
/// The exact fieldless constructor of a closed enum const argument.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ConstVariant {
    pub ty: TypeRef,
    pub declaration: DeclarationRef,
}
/// Canonical bits checked against the type and use site. A v14 named
/// constant may use the payload for tuple fields in logical field order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstValue {
    pub ty: TypeRef,
    pub bits: u128,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Nominal {
        declaration: DeclarationAddress,
        arguments: Range,
    },
    Scalar(Scalar),
    /// Version fourteen source types, separate from the ordinary scalar ABI.
    F16,
    F128,
    /// Version fourteen: the exact local opaque foreign declaration.
    Foreign(DeclarationRef),
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
    /// Version fourteen: one application of a function's return-position
    /// `impl Trait` opaque type, whose captures live in `ColdColumns::opaques`.
    OpaqueResult(OpaqueResultRef),
    Dynamic(DynamicTypeRef),
    /// Version fourteen: an applied closure type whose header lives in `ColdColumns::closures`.
    Closure(ClosureTypeRef),
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
    /// Version thirteen: `[T; N]`. Its length term is held out of line in
    /// [`ColdColumns::array_lengths`], so a type row stays small. Each
    /// consumer proves the element sized from its own facts.
    Array {
        element: TypeRef,
        length: ArrayLengthRef,
    },
    /// Version thirteen: a pattern type over `base`. Each consumer re-proves
    /// the constraint against its own base; no layout or niche fact travels.
    Pattern {
        base: TypeRef,
        constraint: PatternConstraint,
    },
}
/// A trait object names its actual declarations. Its inputs use the inner
/// trait binder, while the object's lifetime belongs to the surrounding frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DynamicTypeRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DynamicType {
    pub principal: Option<DeclarationAddress>,
    pub principal_arguments: u32,
    pub arguments: Range,
    pub bindings: Range,
    pub markers: Range,
    pub binder: BinderRef,
    pub region: RegionRef,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DynamicBinding {
    pub member: DeclarationAddress,
    pub owner: DeclarationAddress,
    pub arguments: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DynamicMarker {
    pub marker: crate::support::marker_traits::AutoTrait,
    pub declaration: DeclarationAddress,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DynamicColumns {
    pub objects: Vec<DynamicType>,
    pub bindings: Vec<DynamicBinding>,
    pub markers: Vec<DynamicMarker>,
}
impl DynamicColumns {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty() && self.bindings.is_empty() && self.markers.is_empty()
    }
}

/// A transported pattern constraint. A range's endpoints are held out of line
/// in [`ColdColumns::pattern_ranges`], so a type row stays small.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PatternConstraint {
    /// `!null`, over a raw pointer base.
    NonNull,
    /// A canonical inclusive scalar interval, over an integer or char base.
    Range(PatternRangeRef),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PatternRangeRef(pub u32);
/// An array type row's length term in [`ColdColumns::array_lengths`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ArrayLengthRef(pub u32);
/// A canonical inclusive interval in raw target-width bits. A signed interval
/// whose start follows its end in value order wraps through the base's
/// maximum, as a connected `A | B` alternative does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PatternRange {
    pub start: u128,
    pub end: u128,
}
/// Version thirteen's transported diagnostic identities. The table is closed:
/// each admitted spelling is a separate, reviewed admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiagnosticRole {
    NonNull,
    /// Core's `Send` is a diagnostic item, not a lang item; consumers install
    /// it into their own `LangItem::Send` slot.
    Send,
    /// Version fourteen binds the actual `Eq` supertrait of `ConstParamTy`.
    Eq,
    /// Version fourteen binds the actual trait required by `panic_display`.
    Display,
    /// The actual enum owner of the Result variant roles.
    Result,
}
impl DiagnosticRole {
    pub const ALL: [Self; 5] = [
        Self::NonNull,
        Self::Send,
        Self::Eq,
        Self::Display,
        Self::Result,
    ];
    #[must_use]
    pub const fn spelling(self) -> &'static str {
        match self {
            Self::NonNull => "NonNull",
            Self::Send => "Send",
            Self::Eq => "Eq",
            Self::Display => "Display",
            Self::Result => "Result",
        }
    }
    #[must_use]
    pub fn from_spelling(spelling: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|role| role.spelling() == spelling)
    }
}
/// A diagnostic identity row. It follows every language role row in the role
/// table, outside the policy's role range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DiagnosticBinding {
    pub role: DiagnosticRole,
    pub declaration: DeclarationAddress,
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
    /// The actual defining trait permits authenticated foreign inherent methods.
    pub const INCOHERENT_INHERENT: u16 = 512;
    pub const SKIP_ARRAY_DISPATCH: u16 = 1024;
    pub const SKIP_BOXED_SLICE_DISPATCH: u16 = 2048;
    /// An empty marker declaration verified during metadata validation.
    pub const MARKER: u16 = 4096;
    /// The checked unsafe specialization policy on the actual `TrivialClone` declaration.
    pub const UNSAFE_SPECIALIZATION: u16 = 8192;
    pub(crate) const DISPATCH_POLICIES: u16 =
        Self::SKIP_ARRAY_DISPATCH | Self::SKIP_BOXED_SLICE_DISPATCH;
    pub(crate) const DECLARATION_POLICIES: u16 = Self::FUNDAMENTAL
        | Self::DYN_INCOMPATIBLE
        | Self::INCOHERENT_INHERENT
        | Self::DISPATCH_POLICIES
        | Self::MARKER
        | Self::UNSAFE_SPECIALIZATION;
    /// # Errors
    /// Rejects claims outside this exact supported structural vocabulary.
    pub fn from_bits(bits: u16) -> Result<Self, &'static str> {
        if bits & !16383 != 0 {
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
    /// Version fourteen retains the provider's checked Core authorization for this owner.
    pub core_primitive_inherent: bool,
    pub constant: bool,
    pub polarity: crate::support::polarity::ImplPolarity,
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
    Intrinsic(BodylessIntrinsic),
}
/// Checked bodyless Rust intrinsics in the version-fourteen profile.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BodylessIntrinsic {
    CallerLocation = 0,
    SizeOf = 1,
    AlignOf = 2,
    PtrMetadata = 3,
    OffsetOf = 4,
    FieldOffset = 5,
}
impl BodylessIntrinsic {
    pub const ALL: [Self; 6] = [
        Self::CallerLocation,
        Self::SizeOf,
        Self::AlignOf,
        Self::PtrMetadata,
        Self::OffsetOf,
        Self::FieldOffset,
    ];

    #[must_use]
    pub const fn kind(self) -> crate::support::bindings::SourceIntrinsic {
        use crate::support::bindings::{LayoutIntrinsic, SourceIntrinsic};
        match self {
            Self::CallerLocation => SourceIntrinsic::CallerLocation,
            Self::SizeOf => SourceIntrinsic::Layout(LayoutIntrinsic::SizeOf),
            Self::AlignOf => SourceIntrinsic::Layout(LayoutIntrinsic::AlignOf),
            Self::PtrMetadata => SourceIntrinsic::PtrMetadata,
            Self::OffsetOf => SourceIntrinsic::OffsetOf,
            Self::FieldOffset => SourceIntrinsic::FieldOffset,
        }
    }

    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::CallerLocation => "caller_location",
            Self::SizeOf => "size_of",
            Self::AlignOf => "align_of",
            Self::PtrMetadata => "ptr_metadata",
            Self::OffsetOf => "offset_of",
            Self::FieldOffset => "field_offset",
        }
    }
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
pub struct ExactCallable {
    pub declaration: DeclarationRef,
    /// Existing exported originals own bodies; version fourteen may name a
    /// checked external runtime declaration instead.
    pub body: Option<BodyRef>,
    /// An independently checked Rust-ABI reference to the user panic handler.
    pub weak_panic: bool,
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
/// Extensibility at use sites outside the defining crate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Exhaustiveness {
    #[default]
    Closed,
    Open,
}
/// An explicit alignment or packing unit on a record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutModifier {
    #[default]
    Natural,
    Aligned(LayoutUnit),
    Packed(LayoutUnit),
}
/// A checked power-of-two byte count stored as its exponent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutUnit(u8);
impl LayoutUnit {
    pub(crate) fn new(bytes: u32) -> Result<Self, &'static str> {
        let invalid = "invalid source nominal alignment or packing unit";
        if !bytes.is_power_of_two() || bytes > 1 << 29 {
            return Err(invalid);
        }
        Ok(Self(
            u8::try_from(bytes.trailing_zeros()).map_err(|_| invalid)?,
        ))
    }
    pub(crate) const fn bytes(self) -> u32 {
        1 << self.0
    }
}
/// Layout and argument behavior distinct from an ordinary record.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NominalForm {
    #[default]
    Ordinary,
    IndirectArguments,
    Union,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NominalLayout {
    pub modifier: LayoutModifier,
    pub form: NominalForm,
}
impl NominalLayout {
    pub(crate) fn validate(
        self,
        repr: crate::support::crates::RecordRepr,
    ) -> Result<(), &'static str> {
        use crate::support::crates::RecordRepr;
        if self.modifier != LayoutModifier::Natural
            && !matches!(repr, RecordRepr::Rust | RecordRepr::C)
            || self.form == NominalForm::IndirectArguments && repr != RecordRepr::C
            || self.form == NominalForm::Union && repr == RecordRepr::Simd
        {
            return Err("source layout modifier requires a compatible nominal representation");
        }
        Ok(())
    }
}
/// Raw, unpublished nominal header rows.
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
    pub layout: NominalLayout,
    /// Locality policy on the defining struct, independent of its representation.
    pub fundamental: bool,
    pub incoherent_inherent: bool,
    /// Packed alignment one on the actual SIMD representation.
    pub packed_simd: bool,
    /// The nominal is open to outside crates even when all current fields or variants are known.
    pub exhaustiveness: Exhaustiveness,
    /// The defining struct's checked monomorphization lane limit.
    pub simd_lane_limit: Option<u64>,
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
    pub exact_callables: Vec<ExactCallable>,
}
pub type SourceRows = Rows;
/// Cold column groups held out of line so packets without them, and every
/// hot record, pay no inline storage: version thirteen's associated-result
/// definitions and enum variant tables, plus the validated sparse original
/// index of a packet with compiler operations (absent from raw rows).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeclaredIntrinsic {
    pub declaration: DeclarationRef,
    pub source_fallback: bool,
}
/// Version fourteen: sparse per-function const-policy annotations outside the
/// hot function header row (`#[rustc_comptime]` or `#[rustc_do_not_const_check]`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionPolicyKind {
    Comptime,
    UncheckedConstBody,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionPolicy {
    pub declaration: DeclarationRef,
    pub kind: FunctionPolicyKind,
}
/// Version fourteen: sealed provider call-site coordinates for non-tracked
/// original bodies whose calls target `#[track_caller]` or `caller_location`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallerSiteRow {
    pub body: BodyRef,
    pub value: u32,
    pub file: u32,
    pub line: u32,
    pub column: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CallerSiteColumns {
    pub files: Vec<Box<str>>,
    pub rows: Vec<CallerSiteRow>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ColdColumns {
    pub associated_definitions: Vec<AssociatedDefinition>,
    pub associated_definition_inputs: Vec<AssociatedDefinitionInputs>,
    pub enums: EnumColumns,
    /// Version thirteen: each range pattern's endpoints, in type-row order.
    pub pattern_ranges: Vec<PatternRange>,
    pub dynamics: DynamicColumns,
    /// Version thirteen: diagnostic identities, after the language roles.
    pub diagnostics: Vec<DiagnosticBinding>,
    /// Version thirteen: each const argument's term, in argument-row order.
    pub consts: Vec<ConstTerm>,
    /// Version thirteen: each array type row's length term, in type-row order.
    pub array_lengths: Vec<ConstTerm>,
    /// Version thirteen: one header row per constant declaration, in
    /// declaration order. Each row holds its own term in place.
    pub constants: Vec<ConstantHeader>,
    pub promoted_handlers: Vec<PromotedHandler>,
    pub type_aliases: Vec<TypeAliasHeader>,
    pub alias_normalizations: Vec<AliasNormalization>,
    pub nominal_region_bounds: Vec<NominalRegionBounds>,
    pub const_defaults: Vec<ConstDefault>,
    pub trait_aliases: Vec<TraitAliasHeader>,
    pub declared_intrinsics: Vec<DeclaredIntrinsic>,
    pub function_policies: Vec<FunctionPolicy>,
    pub opaques: OpaqueColumns,
    pub statics: Option<Box<StaticColumns>>,
    pub caller_sites: Option<Box<CallerSiteColumns>>,
    pub closures: Option<Box<ClosureColumns>>,
    pub(crate) bodies: Option<std::sync::Arc<bodymap::Lookup>>,
}
static EMPTY_COLD: ColdColumns = ColdColumns {
    associated_definitions: Vec::new(),
    associated_definition_inputs: Vec::new(),
    enums: EnumColumns {
        headers: Vec::new(),
        variants: Vec::new(),
        fields: Vec::new(),
        names: Vec::new(),
    },
    pattern_ranges: Vec::new(),
    dynamics: DynamicColumns {
        objects: Vec::new(),
        bindings: Vec::new(),
        markers: Vec::new(),
    },
    diagnostics: Vec::new(),
    consts: Vec::new(),
    array_lengths: Vec::new(),
    constants: Vec::new(),
    promoted_handlers: Vec::new(),
    type_aliases: Vec::new(),
    alias_normalizations: Vec::new(),
    nominal_region_bounds: Vec::new(),
    const_defaults: Vec::new(),
    trait_aliases: Vec::new(),
    declared_intrinsics: Vec::new(),
    function_policies: Vec::new(),
    opaques: OpaqueColumns {
        applications: Vec::new(),
        declarations: Vec::new(),
    },
    statics: None,
    caller_sites: None,
    closures: None,
    bodies: None,
};
impl ColdColumns {
    pub(crate) fn of(cold: Option<&Self>) -> &Self {
        cold.unwrap_or(&EMPTY_COLD)
    }
    #[must_use]
    pub(crate) fn definition_inputs(&self, index: u32) -> Option<Range> {
        let position = self
            .associated_definition_inputs
            .binary_search_by_key(&index, |row| row.index)
            .ok()?;
        Some(self.associated_definition_inputs[position].inputs)
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.associated_definitions.is_empty()
            && self.associated_definition_inputs.is_empty()
            && self.enums.is_empty()
            && self.pattern_ranges.is_empty()
            && self.dynamics.is_empty()
            && self.diagnostics.is_empty()
            && self.consts.is_empty()
            && self.array_lengths.is_empty()
            && self.constants.is_empty()
            && self.promoted_handlers.is_empty()
            && self.type_aliases.is_empty()
            && self.alias_normalizations.is_empty()
            && self.nominal_region_bounds.is_empty()
            && self.const_defaults.is_empty()
            && self.trait_aliases.is_empty()
            && self.declared_intrinsics.is_empty()
            && self.function_policies.is_empty()
            && self.opaques.is_empty()
            && self.statics.is_none()
            && self.caller_sites.is_none()
            && self.closures.is_none()
            && self.bodies.is_none()
    }
}
/// A v14 module alias and its transparent target recipe.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TypeAliasHeader {
    pub declaration: DeclarationRef,
    pub target: TypeRef,
}

/// An ordered deferred projection under its actual transparent alias scope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AliasNormalization {
    pub declaration: DeclarationRef,
    pub target: TypeRef,
}

/// The written outlives obligations of one actual nominal declaration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NominalRegionBounds {
    pub declaration: DeclarationRef,
    pub bounds: Range,
}

/// A direct ordinary alias retains its own lexical Self and canonical clauses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TraitAliasHeader {
    pub declaration: DeclarationRef,
    pub self_definition: DeclarationRef,
    pub self_type: TypeRef,
    pub inputs: Range,
}

impl Rows {
    pub(crate) fn bodyless_runtime(
        &self,
        declaration: DeclarationRef,
        work: &mut crate::support::crates::generics::Work,
    ) -> Result<bool, &'static str> {
        work.charge(1 + self.exact_callables.len().checked_ilog2().unwrap_or(0) as usize)?;
        Ok(self
            .exact_callables
            .binary_search_by_key(&declaration.0, |row| row.declaration.0)
            .is_ok_and(|index| self.exact_callables[index].body.is_none()))
    }

    pub(crate) fn declared_intrinsic(
        &self,
        declaration: DeclarationRef,
        work: &mut crate::support::crates::generics::Work,
    ) -> Result<Option<DeclaredIntrinsic>, &'static str> {
        let rows = &self.cold().declared_intrinsics;
        work.charge(1 + rows.len().checked_ilog2().unwrap_or(0) as usize)?;
        Ok(rows
            .binary_search_by_key(&declaration.0, |row| row.declaration.0)
            .ok()
            .map(|index| rows[index]))
    }

    pub(crate) fn is_comptime(&self, declaration: DeclarationRef) -> bool {
        let rows = &self.cold().function_policies;
        rows.binary_search_by_key(&declaration.0, |row| row.declaration.0)
            .is_ok_and(|index| rows[index].kind == FunctionPolicyKind::Comptime)
    }
    pub(crate) fn is_unchecked_const_body(&self, declaration: DeclarationRef) -> bool {
        let rows = &self.cold().function_policies;
        rows.binary_search_by_key(&declaration.0, |row| row.declaration.0)
            .is_ok_and(|index| rows[index].kind == FunctionPolicyKind::UncheckedConstBody)
    }

    pub(crate) fn type_aliases(&self) -> &[TypeAliasHeader] {
        &self.cold().type_aliases
    }

    pub(crate) fn trait_aliases(&self) -> &[TraitAliasHeader] {
        &self.cold().trait_aliases
    }
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
    #[must_use]
    pub fn pattern_ranges(&self) -> &[PatternRange] {
        &self.cold().pattern_ranges
    }
    #[must_use]
    pub fn diagnostics(&self) -> &[DiagnosticBinding] {
        &self.cold().diagnostics
    }
    #[must_use]
    pub fn consts(&self) -> &[ConstTerm] {
        &self.cold().consts
    }
    #[must_use]
    pub fn array_lengths(&self) -> &[ConstTerm] {
        &self.cold().array_lengths
    }
    #[must_use]
    pub fn closures(&self) -> Option<&ClosureColumns> {
        self.cold().closures.as_deref()
    }
    #[must_use]
    pub fn constants(&self) -> &[ConstantHeader] {
        &self.cold().constants
    }
    /// Each constant declaration with its header row, in declaration order.
    pub fn constant_rows(&self) -> impl Iterator<Item = (DeclarationRef, &ConstantHeader)> {
        free_constants::declared(&self.declarations, self.constants())
    }
    /// The cold groups, allocated on first write. A writer accounts for
    /// `size_of::<ColdColumns>()` before the first call.
    pub fn cold_mut(&mut self) -> &mut ColdColumns {
        self.cold.get_or_insert_with(Box::default)
    }
}
