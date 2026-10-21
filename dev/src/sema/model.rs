//! Declaration and node facts shared by type checking and later body construction.
use crate::{
    lex::token::Punctuation,
    literal::value::FloatBits,
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Namespace, Target},
        package::Error,
        unit::NodeKey,
    },
    syntax::ast::{Kind, List, NodeId, Unary},
    types::{RegionId, Store, TypeId},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Scalar {
    Integer(u128),
    Float(FloatBits),
    Boolean(bool),
    Character(char),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cast {
    FunctionPointer,
    Pointer,
    PointerThin,
    PointerToInteger,
    IntegerToPointer,
    IntegerToInteger,
    IntegerToFloat,
    FloatToInteger,
    FloatToFloat,
    BoolToInteger,
    CharToInteger,
    U8ToChar,
    EnumToInteger,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    Borrow,
    Address,
    AddressDereference,
    PointerAddress {
        operand: NodeId,
    },
    Dereference,
    IndexSlice,
    Construct {
        variant: super::members::VariantId,
    },
    FieldInit {
        field: super::members::FieldId,
    },
    Field {
        field: super::members::Selection,
    },
    Index {
        length: u64,
    },
    IndexParameter {
        length: crate::types::arguments::ConstId,
    },
    AssignPlace {
        operator: Option<Punctuation>,
        left: TypeId,
        right: TypeId,
    },
    /// A break or continue target is in the same source unit as the transfer.
    Transfer {
        target: NodeId,
        continuing: bool,
    },
    Unary {
        operator: Unary,
        operand: TypeId,
        literal: bool,
    },
    Binary {
        operator: Punctuation,
        left: TypeId,
        right: TypeId,
    },
    Cast {
        kind: Cast,
        source: TypeId,
        target: TypeId,
    },
    Assign {
        target: DefId,
        operator: Option<Punctuation>,
        left: TypeId,
        right: TypeId,
    },
    Call {
        signature: TypeId,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdjustmentKind {
    Never,
    FunctionPointer,
    ArrayToSlice,
    Unsize,
    Reborrow,
    Pointer,
    PointerReborrow,
    /// The same type shape, with a directed lifetime relation retained separately.
    Equality,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adjustment {
    pub kind: AdjustmentKind,
    pub target: TypeId,
}
/// Selected built-in reference conversion; kept sparse outside ordinary node facts.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceConversion {
    pub dereferences: u32,
    pub unsize: bool,
    pub two_phase: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReferenceAdjustment {
    pub conversion: ReferenceConversion,
    pub borrow: TypeId,
}
#[derive(Clone, Copy, Default)]
pub struct NodeFacts {
    pub ty: Option<TypeId>,
    pub scalar: Option<Scalar>,
    pub operation: Option<Operation>,
    pub adjustment: Option<Adjustment>,
    pub reference: Option<Target>,
    pub binding: Option<DefId>,
}
#[derive(Clone, Copy)]
pub struct Function {
    pub definition: DefId,
    pub node: NodeKey,
    pub signature: TypeId,
    pub parameters: List,
    pub body: Option<NodeKey>,
}
#[derive(Clone, Copy)]
pub enum InitializerKind {
    Const,
    Static { mutable: bool },
}
#[derive(Clone, Copy)]
pub struct Initializer {
    pub(super) typing: InitializerTyping,
    pub definition: DefId,
    pub node: NodeKey,
    pub value: NodeKey,
    pub ty: TypeId,
    pub kind: InitializerKind,
}
#[derive(Clone, Copy)]
pub(super) enum InitializerTyping {
    Unseen,
    Active,
    Complete,
}
impl Initializer {
    pub(super) fn type_checked(&self) -> bool {
        matches!(self.typing, InitializerTyping::Complete)
    }
}
#[derive(Clone, Copy)]
pub struct RegionEquality {
    pub body: NodeKey,
    pub left: RegionId,
    pub right: RegionId,
}
#[derive(Clone, Copy)]
pub struct Outlives {
    pub body: NodeKey,
    pub source: NodeKey,
    pub longer: RegionId,
    pub shorter: RegionId,
}
/// Shared proof scopes keep diagnostic origins separate from lifetime authority.
#[derive(Clone)]
pub struct LifetimeProof {
    pub source: NodeKey,
    pub sources: Vec<NodeKey>,
    pub proof: crate::types::lifetime_proof::Obligations,
}
impl LifetimeProof {
    #[must_use]
    pub fn new(source: NodeKey, proof: crate::types::lifetime_proof::Obligations) -> Self {
        Self {
            source,
            sources: Vec::new(),
            proof,
        }
    }
}
pub struct QuantifiedProof {
    pub body: NodeKey,
    pub evidence: LifetimeProof,
}
#[derive(Clone, Copy)]
pub struct TypeOutlives {
    pub body: NodeKey,
    pub source: NodeKey,
    pub ty: TypeId,
    pub shorter: RegionId,
}
/// A selected conversion awaiting variance-aware region checking. Keep all
/// coercion sites: one syntax node can constrain several inferred targets.
#[derive(Clone, Copy)]
pub struct Coercion {
    pub body: NodeKey,
    pub source: NodeKey,
    pub actual: TypeId,
    pub expected: TypeId,
    pub kind: AdjustmentKind,
    pub reference: Option<ReferenceConversion>,
}
/// Types used in a body must satisfy their well-formedness bounds. Function
/// inputs/outputs supply assumptions; local types never add assumptions.
#[derive(Clone, Copy)]
pub struct WellFormed {
    pub body: NodeKey,
    pub source: NodeKey,
    pub ty: TypeId,
}
/// Declaration elaboration resolves binders even when a local type has holes.
/// Only the unresolved shape is completed by the owning body's inference graph.
#[derive(Clone, Copy)]
pub enum PartialAnnotation {
    Hole,
    Shape,
    Reference(RegionId),
    Function { bound_regions: u32 },
}
#[derive(Clone, Copy)]
pub enum BodyRegionOrigin {
    QuantifiedProof,
    Annotation,
    Inference,
    ItemArgument,
    CallArgument,
    UniversalAlias,
    ExpressionBorrow,
    PatternBorrow,
    PatternInput,
}
/// Existential region identity `1 + ordinal`; these are obligations, not proofs.
#[derive(Clone, Copy)]
pub struct BodyRegion {
    pub body: NodeKey,
    pub source: NodeKey,
    pub origin: BodyRegionOrigin,
}
/// Deferred builtin uses retain their source and exact caller environment until
/// the body's lifetime graph can establish identity of inferred arguments.
#[derive(Clone)]
pub struct BuiltinRequirements {
    pub(crate) environment:
        Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
    pub(crate) predicates: Vec<(NodeKey, crate::types::parameters::predicates::Predicate)>,
}
#[derive(Clone, Copy)]
pub struct ReferenceAlignment {
    pub source: NodeKey,
    pub ty: TypeId,
    pub packed: u32,
    pub guard: bool,
}
/// Work observations are not language facts and are never part of a text oracle.
#[derive(Default)]
pub struct BodyWork {
    /// Actual typed unsizing DAG inputs and reuse, separate from eligibility queries.
    pub unsizing_inputs: usize,
    pub unsizing_fields: usize,
    pub inference_steps: usize,
    /// Completed canonical inference and selection work, including suspended searches.
    pub canonical_steps: usize,
    /// Actual ordinary trait goal attempts, including resumed retained candidates.
    pub trait_selections: usize,
    /// Field storage preparation uses its own cursor after consuming inference.
    pub storage_waits: usize,
    pub storage_definition_waits: usize,
    pub storage_selections: usize,
    pub storage_steps: usize,
    /// Exact definition dependencies yielded by owned body projection queries.
    pub projection_waits: usize,
    /// Cursor resolver requests and isolated partial-input selection attempts.
    pub projection_selections: usize,
    /// Type-fold work reported by owned body normalization cursors.
    pub projection_steps: usize,
    pub steps: usize,
    pub suspensions: usize,
    pub suspended_slots: usize,
    pub peak_suspended_slots: usize,
}
/// Actual dispatch operations, excluding selection elsewhere in the frontend.
/// These observations are never serialized into course text oracles.
#[derive(Default)]
pub struct DispatchWork {
    pub selections: usize,
    pub shared_selections: usize,
    pub closed_proofs: usize,
    pub dependency_expansions: usize,
    /// Typed initializer edges visited to demand checked constant members.
    pub member_dependencies: usize,
}
/// Shared field-graph work is independent of each caller's proof traversal.
#[derive(Default)]
pub struct StoredFieldWork {
    pub steps: usize,
    pub recipes: usize,
    /// Charged recipe storage budget, not a measurement of allocator overhead.
    pub retained_bytes: usize,
}
pub struct Declarations {
    /// Const permission belongs to a selected declaration, not its callable type.
    pub(crate) constant_functions: std::collections::HashSet<DefId>,
    pub(super) target_feature_functions: std::collections::HashSet<DefId>,
    pub closures: super::closures::Bodies,
    pub opaques: super::opaque::Opaques,
    pub body_work: BodyWork,
    pub alias_requirements: super::alias_requirements::Requirements,
    pub unreduced: super::unreduced::Types,
    pub(super) anonymous_constants: std::collections::HashMap<NodeKey, TypeId>,
    pub(super) constant_annotation_bodies: std::collections::HashSet<NodeKey>,
    pub generics: super::generics::Generics,
    /// Repeat operands selected as constant paths before runtime coercion wrappers.
    pub constant_repeats: std::collections::HashSet<NodeKey>,
    pub(crate) constant_targets:
        std::collections::HashMap<super::values::Instance, super::values::Instance>,
    pub(crate) dispatch_dependencies: super::dispatch::Dependencies,
    pub(crate) objects: super::vtable_plans::Objects,
    pub(crate) constant_dependencies:
        std::collections::HashMap<DefId, Vec<super::values::Dependency>>,
    pub constant_uses: std::collections::HashMap<NodeKey, super::values::Instance>,
    pub(super) constant_instances:
        std::collections::HashMap<super::values::Instance, super::constants::Value>,
    pub associated: super::associated::Inventory,
    pub(crate) stored_fields: Option<Box<super::stored_fields::Registry>>,
    pub methods: super::methods::Methods,
    pub(crate) invocations: std::collections::HashMap<NodeKey, super::invocations::Invocation>,
    pub receivers: std::collections::HashMap<NodeKey, super::associated::Receiver>,
    pub destructors: super::destructors::Destructors,
    /// Packed source places remain checked even when their CFG is unreachable.
    pub reference_alignment: Vec<ReferenceAlignment>,
    pub reference_adjustments: std::collections::HashMap<NodeKey, ReferenceAdjustment>,
    pub(crate) unsizing_rules: Option<Box<crate::types::unsizing::Rules>>,
    /// Selected terminal conversion; lowering consumes its retained proof inputs.
    pub unsizing_adjustments:
        std::collections::HashMap<NodeKey, crate::types::unsizing::Conversion<TypeId>>,
    /// Raw pointer type before a separately selected storage-tail unsizing.
    pub pointer_adjustments: std::collections::HashMap<NodeKey, TypeId>,
    pub partial_annotations: std::collections::HashMap<NodeKey, PartialAnnotation>,
    pub well_formed: Vec<WellFormed>,
    pub coercions: Vec<Coercion>,
    pub outlives: Vec<Outlives>,
    pub type_outlives: Vec<TypeOutlives>,
    pub quantified_proofs: Vec<QuantifiedProof>,
    pub lifetimes: super::lifetimes::Lifetimes,
    pub(super) constant_values: std::collections::HashMap<DefId, super::constants::Value>,
    pub(super) constant_products: super::constants::products::Products,
    pub(super) constant_references: super::constants::references::References,
    pub autoderefs: std::collections::HashMap<NodeKey, u32>,
    pub texts: super::text::Texts,
    pub text_literals: std::collections::HashMap<NodeKey, super::text::Id>,
    pub patterns: super::patterns::Patterns,
    pub enum_casts: std::collections::HashSet<crate::types::ItemId>,
    pub derives: std::collections::HashMap<crate::types::ItemId, crate::types::copy::Properties>,
    pub equality_derives:
        std::collections::HashMap<crate::types::ItemId, crate::types::equality::Properties>,
    pub equality: crate::types::equality::Equality,
    pub copies: crate::types::copy::Copies,
    pub aggregates: crate::types::aggregate::Definitions,
    pub members: super::members::Members,
    pub layouts: crate::abi::layout::Layouts,
    pub region_equalities: Vec<RegionEquality>,
    pub body_regions: Vec<BodyRegion>,
    pub initializers: Vec<Initializer>,
    pub types: Store,
    pub definitions: Vec<Option<TypeId>>,
    pub nodes: Vec<Vec<NodeFacts>>,
    pub functions: Vec<Function>,
}
impl Declarations {
    #[must_use]
    pub fn is_const_function(&self, definition: DefId) -> bool {
        self.constant_functions.contains(&definition)
    }
    /// Constant expressions with lifetime obligations retain their own proof body.
    /// Sorting is confined to consumers that publish a deterministic body order.
    #[must_use]
    pub fn constant_bodies(&self) -> Vec<(NodeKey, TypeId)> {
        let mut bodies: Vec<_> = self
            .anonymous_constants
            .iter()
            .map(|(&key, &ty)| (key, ty))
            .collect();
        bodies.sort_unstable_by_key(|(key, _)| (key.file.0, key.node.0));
        bodies
    }
    pub(super) fn lifetime_fact_count(&self) -> usize {
        self.body_regions.len()
            + self.region_equalities.len()
            + self.coercions.len()
            + self.outlives.len()
            + self.type_outlives.len()
            + self.quantified_proofs.len()
            + self.well_formed.len()
    }
    /// Retain the directed lifetime relation independently of runtime adjustment
    /// selection. In particular an explicit cast must not reify its operand twice.
    pub(super) fn retain_coercion(&mut self, coercion: Coercion) {
        if coercion.actual != coercion.expected
            && coercion.kind != AdjustmentKind::Never
            && (self.types.contains_regions(coercion.actual) == Some(true)
                || self.types.contains_regions(coercion.expected) == Some(true))
        {
            self.coercions.push(coercion);
        }
    }
    /// Read-only counters for dispatch selection and dependency traversal.
    #[must_use]
    pub fn dispatch_work(&self) -> &DispatchWork {
        &self.dispatch_dependencies.work
    }
    /// Read-only, nonserialized structural storage observations.
    #[must_use]
    pub fn stored_field_work(&self) -> StoredFieldWork {
        self.stored_fields.as_deref().map_or_else(
            StoredFieldWork::default,
            super::stored_fields::Registry::work,
        )
    }
    pub(super) fn new(report: &Report) -> Result<Self, Error> {
        let mut value = Self {
            constant_functions: std::collections::HashSet::new(),
            target_feature_functions: std::collections::HashSet::new(),
            closures: super::closures::Bodies::default(),
            opaques: super::opaque::Opaques::default(),
            body_work: BodyWork::default(),
            alias_requirements: super::alias_requirements::Requirements::default(),
            unreduced: super::unreduced::Types::default(),
            anonymous_constants: std::collections::HashMap::new(),
            constant_annotation_bodies: std::collections::HashSet::new(),
            generics: super::generics::Generics::default(),
            constant_repeats: std::collections::HashSet::new(),
            constant_targets: std::collections::HashMap::new(),
            dispatch_dependencies: super::dispatch::Dependencies::default(),
            objects: super::vtable_plans::Objects::default(),
            constant_dependencies: std::collections::HashMap::new(),
            constant_uses: std::collections::HashMap::new(),
            constant_instances: std::collections::HashMap::new(),
            associated: super::associated::collect(report)?,
            stored_fields: None,
            methods: super::methods::Methods::default(),
            invocations: std::collections::HashMap::new(),
            receivers: std::collections::HashMap::new(),
            destructors: super::destructors::Destructors::default(),
            reference_alignment: Vec::new(),
            reference_adjustments: std::collections::HashMap::new(),
            unsizing_rules: None,
            unsizing_adjustments: std::collections::HashMap::new(),
            pointer_adjustments: std::collections::HashMap::new(),
            partial_annotations: std::collections::HashMap::new(),
            well_formed: Vec::new(),
            type_outlives: Vec::new(),
            quantified_proofs: Vec::new(),
            coercions: Vec::new(),
            outlives: Vec::new(),
            lifetimes: super::lifetimes::Lifetimes::default(),
            constant_values: std::collections::HashMap::new(),
            constant_references: super::constants::references::References::default(),
            autoderefs: std::collections::HashMap::new(),
            constant_products: super::constants::products::Products::default(),
            texts: super::text::Texts::default(),
            text_literals: std::collections::HashMap::new(),
            patterns: super::patterns::Patterns::default(),
            enum_casts: std::collections::HashSet::new(),
            derives: std::collections::HashMap::new(),
            equality_derives: std::collections::HashMap::new(),
            equality: crate::types::equality::Equality::default(),
            copies: crate::types::copy::Copies::default(),
            aggregates: crate::types::aggregate::Definitions::default(),
            members: super::members::Members::default(),
            layouts: crate::abi::layout::Layouts::default(),
            region_equalities: Vec::new(),
            body_regions: Vec::new(),
            initializers: Vec::new(),
            types: Store::default(),
            definitions: vec![None; report.resolution.definitions.len()],
            nodes: report
                .package
                .units
                .iter()
                .map(|unit| {
                    vec![
                        NodeFacts::default();
                        if unit.include_context.is_none() {
                            unit.configured.ast.nodes.len()
                        } else {
                            0
                        }
                    ]
                })
                .collect(),
            functions: Vec::new(),
        };
        value.import_resolution(report)?;
        Ok(value)
    }
    fn import_resolution(&mut self, report: &Report) -> Result<(), Error> {
        for reference in &report.resolution.references {
            if reference.namespace != Namespace::Macro {
                let facts = self.node_mut(reference.node).ok_or_else(|| {
                    Self::error(
                        report,
                        reference.node,
                        "invalid resolved reference identity",
                    )
                })?;
                facts.reference = Some(reference.target);
            }
        }
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            if (matches!(
                definition.kind,
                DefKind::Local | DefKind::Parameter | DefKind::Label | DefKind::Generic
            ) || definition.namespace == Namespace::Lifetime)
                && let Some(key) = definition.node
            {
                let id = DefId(
                    u32::try_from(index)
                        .map_err(|_| Self::error(report, key, "too many definitions"))?,
                );
                self.node_mut(key)
                    .ok_or_else(|| Self::error(report, key, "invalid binding node"))?
                    .binding = Some(id);
            }
        }
        for &(key, definition) in &report.resolution.pattern_aliases {
            self.node_mut(key)
                .ok_or_else(|| Self::error(report, key, "invalid pattern binding alias"))?
                .binding = Some(definition);
        }
        Ok(())
    }
    #[must_use]
    pub fn node(&self, key: NodeKey) -> Option<&NodeFacts> {
        self.nodes
            .get(key.file.0 as usize)?
            .get(key.node.0 as usize)
    }
    pub(super) fn node_mut(&mut self, key: NodeKey) -> Option<&mut NodeFacts> {
        self.nodes
            .get_mut(key.file.0 as usize)?
            .get_mut(key.node.0 as usize)
    }
    pub(crate) fn error(report: &Report, key: NodeKey, message: impl Into<String>) -> Error {
        Error::at(
            key.file,
            report
                .package
                .unit(key.file)
                .configured
                .ast
                .node(key.node)
                .span
                .start,
            message,
        )
    }
    pub(super) fn kind(report: &Report, key: NodeKey) -> Kind {
        report
            .package
            .unit(key.file)
            .configured
            .ast
            .node(key.node)
            .kind
    }
}
