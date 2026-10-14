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
    Reborrow,
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
    pub(super) type_checked: bool,
    pub definition: DefId,
    pub node: NodeKey,
    pub value: NodeKey,
    pub ty: TypeId,
    pub kind: InitializerKind,
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
#[derive(Clone, Copy)]
pub struct ReferenceAlignment {
    pub source: NodeKey,
    pub ty: TypeId,
    pub packed: u32,
    pub guard: bool,
}
pub struct Declarations {
    pub(super) anonymous_constants: std::collections::HashMap<NodeKey, TypeId>,
    pub(super) constant_annotation_bodies: std::collections::HashSet<NodeKey>,
    pub generics: super::generics::Generics,
    /// Repeat operands selected as constant paths before runtime coercion wrappers.
    pub constant_repeats: std::collections::HashSet<NodeKey>,
    pub associated: super::associated::Inventory,
    pub receivers: std::collections::HashMap<NodeKey, super::associated::Receiver>,
    pub destructors: super::destructors::Destructors,
    /// Packed source places remain checked even when their CFG is unreachable.
    pub reference_alignment: Vec<ReferenceAlignment>,
    pub reference_adjustments: std::collections::HashMap<NodeKey, ReferenceAdjustment>,
    pub partial_annotations: std::collections::HashMap<NodeKey, PartialAnnotation>,
    pub well_formed: Vec<WellFormed>,
    pub coercions: Vec<Coercion>,
    pub outlives: Vec<Outlives>,
    pub type_outlives: Vec<TypeOutlives>,
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
    pub(super) fn new(report: &Report) -> Result<Self, Error> {
        let mut value = Self {
            anonymous_constants: std::collections::HashMap::new(),
            constant_annotation_bodies: std::collections::HashSet::new(),
            generics: super::generics::Generics::default(),
            constant_repeats: std::collections::HashSet::new(),
            associated: super::associated::collect(report)?,
            receivers: std::collections::HashMap::new(),
            destructors: super::destructors::Destructors::default(),
            reference_alignment: Vec::new(),
            reference_adjustments: std::collections::HashMap::new(),
            partial_annotations: std::collections::HashMap::new(),
            well_formed: Vec::new(),
            type_outlives: Vec::new(),
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
        for reference in &report.resolution.references {
            if reference.namespace != Namespace::Macro {
                let facts = value.node_mut(reference.node).ok_or_else(|| {
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
                value
                    .node_mut(key)
                    .ok_or_else(|| Self::error(report, key, "invalid binding node"))?
                    .binding = Some(id);
            }
        }
        for &(key, definition) in &report.resolution.pattern_aliases {
            value
                .node_mut(key)
                .ok_or_else(|| Self::error(report, key, "invalid pattern binding alias"))?
                .binding = Some(definition);
        }
        Ok(value)
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
