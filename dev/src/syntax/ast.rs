//! Dense syntax identities and child ranges. Nodes never recursively own nodes.
use super::{
    item::{AttributeCapture, AttributeRange, FunctionType, GenericParameter, Item},
    item::{UseTree, Variant},
    token_tree::{Forest, Span},
};
use crate::{lex::token::Punctuation, literal::Value};

/// A syntax node's arena index, stored off by one so `Option<NodeId>` is as
/// small as `NodeId`. Every observable use goes through [`NodeId::get`].
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(std::num::NonZeroU32);

impl NodeId {
    /// The node at `index`; allocators use [`NodeId::checked`] instead.
    #[must_use]
    pub const fn new(index: u32) -> Self {
        match std::num::NonZeroU32::new(index.wrapping_add(1)) {
            Some(value) => Self(value),
            None => Self(std::num::NonZeroU32::MAX),
        }
    }
    /// The node at `index`, or `None` for the one unrepresentable index.
    #[must_use]
    pub const fn checked(index: u32) -> Option<Self> {
        match std::num::NonZeroU32::new(index.wrapping_add(1)) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get() - 1
    }
}

impl std::fmt::Debug for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("NodeId").field(&self.get()).finish()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct List {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unary {
    Negate,
    Not,
    Dereference,
    Borrow,
    BorrowMut,
    RawConst,
    RawMut,
    Try,
    Await,
}

impl Unary {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Negate => "negate",
            Self::Not => "not",
            Self::Dereference => "deref",
            Self::Borrow => "borrow",
            Self::BorrowMut => "borrow-mut",
            Self::RawConst => "raw-const",
            Self::RawMut => "raw-mut",
            Self::Try => "try",
            Self::Await => "await",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockFlavor {
    Plain,
    Unsafe,
    Const,
    Async,
    AsyncMove,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PatternFlavor {
    Tuple,
    Slice,
    Variant,
    Or,
}

/// Compound type shapes. A path type is stored as its path node alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeFlavor {
    Tuple,
    Slice,
    Array,
    Ref,
    RefMut,
    PointerConst,
    PointerMut,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MacroContext {
    Expression,
    Statements,
    Type,
    Pattern,
    Items(super::item::ItemContext),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    LibraryValue {
        kind: crate::support::library_types::LibraryValue,
        value: Option<NodeId>,
    },
    Receiver {
        name: Span,
        lifetime: Span,
        mutable: bool,
    },
    GenericMethodCall {
        receiver: NodeId,
        method: NodeId,
        arguments: List,
    },
    DeferredLiteral(crate::lex::token::Token),
    Variadic,
    Item(u32),
    Crate(List),
    Expansion {
        children: List,
        tail: Option<NodeId>,
    },
    Attributed {
        attributes: AttributeRange,
        value: NodeId,
    },
    Applied {
        base: NodeId,
        arguments: List,
        output: Option<NodeId>,
        parenthesized: bool,
    },
    Qualified {
        ty: NodeId,
        trait_path: Option<NodeId>,
        path: NodeId,
    },
    Constraint {
        target: NodeId,
        value: Option<NodeId>,
        bounds: List,
    },
    Bound {
        optional: bool,
        constness: crate::support::constness::Constness,
        quantified: bool,
        binders: List,
        value: NodeId,
    },
    Generic(u32),
    Where {
        quantified: bool,
        binders: List,
        ty: NodeId,
        bounds: List,
    },
    FunctionType(u32),
    TraitType {
        opaque: bool,
        bounds: List,
    },
    PreciseCapture(List),
    FieldDefinition {
        name: Option<Span>,
        visibility: super::item::Visibility,
        ty: NodeId,
    },
    Variant(u32),
    UseTree(u32),
    StructPattern {
        path: NodeId,
        fields: List,
        rest: bool,
    },
    PatternField {
        name: Span,
        pattern: NodeId,
    },
    PatternRange {
        start: Option<NodeId>,
        end: Option<NodeId>,
        inclusive: bool,
    },
    Literal(u32),
    /// An identifier or lifetime spelling. The parser interns it when it
    /// shares the crate's names; otherwise resolution interns the span.
    Name(Span, Option<crate::support::names::NameId>),
    CrateRoot(u32),
    Path {
        absolute: bool,
        segments: List,
    },
    Group(NodeId),
    Tuple(List),
    Array(List),
    Repeat {
        value: NodeId,
        count: NodeId,
    },
    Unary {
        op: Unary,
        operand: NodeId,
    },
    Binary {
        op: Punctuation,
        left: NodeId,
        right: NodeId,
    },
    Range {
        inclusive: bool,
        start: Option<NodeId>,
        end: Option<NodeId>,
    },
    Cast {
        value: NodeId,
        ty: NodeId,
    },
    Call {
        callee: NodeId,
        arguments: List,
    },
    MethodCall {
        receiver: NodeId,
        name: Span,
        arguments: List,
    },
    Index {
        value: NodeId,
        index: NodeId,
    },
    Field {
        value: NodeId,
        name: Span,
    },
    Struct {
        path: NodeId,
        fields: List,
        base: Option<NodeId>,
    },
    FieldInit {
        name: Span,
        value: NodeId,
    },
    Block {
        flavor: BlockFlavor,
        statements: List,
        tail: Option<NodeId>,
    },
    EmptyStatement,
    Statement(NodeId),
    Let {
        pattern: NodeId,
        ty: Option<NodeId>,
        value: Option<NodeId>,
        otherwise: Option<NodeId>,
    },
    LetCondition {
        pattern: NodeId,
        value: NodeId,
    },
    If {
        condition: NodeId,
        then: NodeId,
        otherwise: Option<NodeId>,
    },
    Loop {
        body: NodeId,
    },
    While {
        condition: NodeId,
        body: NodeId,
    },
    For {
        pattern: NodeId,
        iterator: NodeId,
        body: NodeId,
    },
    Label {
        name: Span,
        value: NodeId,
    },
    Return(Option<NodeId>),
    Break {
        label: Option<Span>,
        value: Option<NodeId>,
    },
    Continue(Option<Span>),
    Match {
        value: NodeId,
        arms: List,
    },
    Arm {
        pattern: NodeId,
        guard: Option<NodeId>,
        value: NodeId,
    },
    Closure {
        parameters: List,
        result: Option<NodeId>,
        body: NodeId,
        moving: bool,
        asynchronous: bool,
    },
    Parameter {
        pattern: NodeId,
        ty: Option<NodeId>,
    },
    Macro {
        path: NodeId,
        capture: u32,
        context: MacroContext,
    },
    Binding {
        name: Span,
        /// The interned name, as for [`Kind::Name`].
        symbol: Option<crate::support::names::NameId>,
        mutable: bool,
        by_ref: bool,
        subpattern: Option<NodeId>,
    },
    Wildcard,
    RestPattern,
    Pattern {
        flavor: PatternFlavor,
        path: Option<NodeId>,
        elements: List,
    },
    ReferencePattern {
        mutable: bool,
        pattern: NodeId,
    },
    Type {
        flavor: TypeFlavor,
        elements: List,
    },
    InferType,
    NeverType,
    /// A parsed implementation domain, independent of the trait's visibility.
    ImplRestriction {
        path: NodeId,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    pub span: Span,
    pub kind: Kind,
}

/// Configuration inputs of an unconfigured arena, each in ascending node ID
/// (creation) order. The parser and fragment transfer append them as nodes
/// are created; configuration consumes them instead of walking the tree.
#[derive(Debug, Default)]
pub struct Pending {
    /// Every `Attributed` attribute owner.
    pub owners: Vec<NodeId>,
    /// Every `DeferredLiteral` token node.
    pub literals: Vec<NodeId>,
}

#[derive(Debug)]
pub struct Syntax<L> {
    pub nodes: Vec<Node>,
    pub items: Vec<Item>,
    pub(crate) gate_sites: Option<Box<super::gate_sites::Declarations>>,
    pub variants: Vec<Variant>,
    pub use_trees: Vec<UseTree>,
    pub generics: Vec<GenericParameter>,
    pub function_types: Vec<FunctionType>,
    pub attributes: Vec<AttributeCapture>,
    pub edges: Vec<NodeId>,
    pub literals: Vec<L>,
    pub captures: Vec<Forest>,
    pub root: NodeId,
    pub lookahead_high_water: usize,
    pub pending: Pending,
}

pub type Ast<'source> = Syntax<Value<'source>>;

impl<L> Default for Syntax<L> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            items: Vec::new(),
            gate_sites: None,
            variants: Vec::new(),
            use_trees: Vec::new(),
            generics: Vec::new(),
            function_types: Vec::new(),
            attributes: Vec::new(),
            edges: Vec::new(),
            literals: Vec::new(),
            captures: Vec::new(),
            root: NodeId::new(0),
            lookahead_high_water: 0,
            pending: Pending::default(),
        }
    }
}

/// A path's written segments. A relative path of one plain name is stored as
/// that `Name` node alone, so its single segment is the path node itself.
#[derive(Clone, Copy, Debug)]
pub enum Segments<'a> {
    List(&'a [NodeId]),
    One([NodeId; 1]),
}

impl std::ops::Deref for Segments<'_> {
    type Target = [NodeId];
    fn deref(&self) -> &[NodeId] {
        match self {
            Self::List(list) => list,
            Self::One(one) => one,
        }
    }
}

impl<L> Syntax<L> {
    #[must_use]
    pub fn list(&self, range: List) -> &[NodeId] {
        &self.edges[range.start as usize..range.end as usize]
    }
    /// Whether a path is absolute and its segments, for a written `Path` or a
    /// compact one-name path; `None` for every other node kind. A `Name` in a
    /// lifetime or shorthand-field position is not a path: callers decide by
    /// syntactic position.
    #[must_use]
    pub fn path_segments(&self, id: NodeId) -> Option<(bool, Segments<'_>)> {
        match self.nodes[id.get() as usize].kind {
            Kind::Path { absolute, segments } => {
                Some((absolute, Segments::List(self.list(segments))))
            }
            Kind::Name(..) => Some((false, Segments::One([id]))),
            _ => None,
        }
    }
    #[must_use]
    pub fn node(&self, id: NodeId) -> Node {
        self.nodes[id.get() as usize]
    }

    /// Change literal ownership while moving every syntax arena intact.
    ///
    /// # Errors
    /// Propagates errors from the supplied literal-storage conversion.
    pub fn map_literals<M, E>(
        self,
        convert: impl FnMut(L) -> Result<M, E>,
    ) -> Result<Syntax<M>, E> {
        Ok(Syntax {
            nodes: self.nodes,
            items: self.items,
            gate_sites: self.gate_sites,
            variants: self.variants,
            use_trees: self.use_trees,
            generics: self.generics,
            function_types: self.function_types,
            attributes: self.attributes,
            edges: self.edges,
            captures: self.captures,
            root: self.root,
            lookahead_high_water: self.lookahead_high_water,
            pending: self.pending,
            literals: self
                .literals
                .into_iter()
                .map(convert)
                .collect::<Result<_, _>>()?,
        })
    }
}
