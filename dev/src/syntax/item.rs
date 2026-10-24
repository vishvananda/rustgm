//! Item-specific payloads stay out of the compact expression-node union.
use super::{
    ast::{List, NodeId},
    attribute::Attribute,
    token_tree::{Forest, Span},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemContext {
    Module,
    Trait,
    Impl,
    External,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Private,
    Public,
    Restricted(NodeId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Unit,
    Tuple,
    Record,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GenericFlavor {
    Lifetime,
    Type,
    Const,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Safety {
    #[default]
    Default,
    Safe,
    Unsafe,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FunctionFlags {
    pub constant: bool,
    pub asynchronous: bool,
    pub safety: Safety,
}

/// Distinguishes an ordinary Rust signature from `extern` with its ABI omitted.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Abi {
    #[default]
    Rust,
    Extern(Option<NodeId>),
}
impl Abi {
    #[must_use]
    pub const fn literal(self) -> Option<NodeId> {
        match self {
            Self::Rust => None,
            Self::Extern(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Function {
        flags: FunctionFlags,
        abi: Abi,
        parameters: List,
        result: Option<NodeId>,
        body: Option<NodeId>,
    },
    Struct {
        shape: Shape,
        fields: List,
    },
    Enum {
        variants: List,
    },
    Union {
        fields: List,
    },
    Trait {
        auto_: bool,
        unsafe_: bool,
        constant: bool,
        bounds: List,
        items: List,
    },
    Impl {
        unsafe_: bool,
        constant: bool,
        negative: bool,
        trait_path: Option<NodeId>,
        ty: NodeId,
        items: List,
    },
    Const {
        ty: NodeId,
        value: Option<NodeId>,
    },
    Static {
        safety: Safety,
        mutable: bool,
        ty: NodeId,
        value: Option<NodeId>,
    },
    TypeAlias {
        bounds: List,
        ty: Option<NodeId>,
    },
    Module {
        items: Option<List>,
    },
    Use(NodeId),
    ExternCrate {
        rename: Option<Span>,
    },
    ExternBlock {
        unsafe_: bool,
        abi: Option<NodeId>,
        items: List,
    },
    MacroRules {
        capture: u32,
    },
    DeclarativeMacro {
        parameters: Option<u32>,
        capture: u32,
    },
    MacroInvocation(NodeId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Item {
    pub name: Option<Span>,
    pub visibility: Visibility,
    pub generics: List,
    pub where_clause: List,
    pub kind: ItemKind,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AttributeRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Clone, Debug)]
pub struct AttributeCapture {
    pub forest: Forest,
    pub attribute: Attribute,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GenericParameter {
    pub flavor: GenericFlavor,
    pub name: Span,
    pub ty: Option<NodeId>,
    pub bounds: List,
    pub default: Option<NodeId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FunctionType {
    pub unsafe_: bool,
    pub abi: Abi,
    pub binders: List,
    pub parameters: List,
    pub result: Option<NodeId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Variant {
    pub name: Span,
    pub shape: Shape,
    pub fields: List,
    pub discriminant: Option<NodeId>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UseTree {
    pub prefix: Option<NodeId>,
    pub children: List,
    pub rename: Option<NodeId>,
    /// The star owns its hygiene context independently of the imported prefix.
    pub glob: Option<Span>,
}
