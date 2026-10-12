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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
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
    /// Type equality was established modulo retained region obligations.
    Equality,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Adjustment {
    pub kind: AdjustmentKind,
    pub target: TypeId,
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
pub struct Declarations {
    pub region_equalities: Vec<RegionEquality>,
    pub initializers: Vec<Initializer>,
    pub types: Store,
    pub definitions: Vec<Option<TypeId>>,
    pub nodes: Vec<Vec<NodeFacts>>,
    pub functions: Vec<Function>,
}
impl Declarations {
    pub(super) fn new(report: &Report) -> Result<Self, Error> {
        let mut value = Self {
            region_equalities: Vec::new(),
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
                DefKind::Local | DefKind::Parameter | DefKind::Label
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
    pub(super) fn error(report: &Report, key: NodeKey, message: impl Into<String>) -> Error {
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
