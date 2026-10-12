//! Item-specific payloads stay out of the compact expression-node union.
use super::{
    ast::{List, NodeId},
    attribute::Attribute,
    token_tree::{Forest, Span},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    Function {
        flags: FunctionFlags,
        abi: Option<NodeId>,
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
        unsafe_: bool,
        bounds: List,
        items: List,
    },
    Impl {
        unsafe_: bool,
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
    pub abi: Option<NodeId>,
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
    pub glob: bool,
}
