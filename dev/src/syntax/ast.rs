//! Dense syntax identities and child ranges. Nodes never recursively own nodes.
use super::{
    item::{AttributeCapture, AttributeRange, FunctionType, GenericParameter, Item},
    item::{UseTree, Variant},
    token_tree::{Forest, Span},
};
use crate::{lex::token::Punctuation, literal::Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TypeFlavor {
    Path,
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
    Name(Span),
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    pub span: Span,
    pub kind: Kind,
}

#[derive(Debug)]
pub struct Syntax<L> {
    pub nodes: Vec<Node>,
    pub items: Vec<Item>,
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
}

pub type Ast<'source> = Syntax<Value<'source>>;

impl<L> Default for Syntax<L> {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            items: Vec::new(),
            variants: Vec::new(),
            use_trees: Vec::new(),
            generics: Vec::new(),
            function_types: Vec::new(),
            attributes: Vec::new(),
            edges: Vec::new(),
            literals: Vec::new(),
            captures: Vec::new(),
            root: NodeId(0),
            lookahead_high_water: 0,
        }
    }
}

impl<L> Syntax<L> {
    #[must_use]
    pub fn list(&self, range: List) -> &[NodeId] {
        &self.edges[range.start as usize..range.end as usize]
    }
    #[must_use]
    pub fn node(&self, id: NodeId) -> Node {
        self.nodes[id.0 as usize]
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
            variants: self.variants,
            use_trees: self.use_trees,
            generics: self.generics,
            function_types: self.function_types,
            attributes: self.attributes,
            edges: self.edges,
            captures: self.captures,
            root: self.root,
            lookahead_high_water: self.lookahead_high_water,
            literals: self
                .literals
                .into_iter()
                .map(convert)
                .collect::<Result<_, _>>()?,
        })
    }
}
