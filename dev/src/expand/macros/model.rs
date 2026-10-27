//! Flat matcher/transcriber syntax and dense binding identities.
use crate::{
    lex::token::Token,
    support::names::NameId,
    syntax::{
        expression::fragment::FragmentKind,
        token_tree::{Delimiter, Span},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct NodeId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VariableId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct List {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repetition {
    ZeroOrMore,
    OneOrMore,
    Optional,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fragment {
    Syntax(FragmentKind),
    Identifier,
    Lifetime,
    Tree,
    Metadata,
    Visibility,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Atom {
    pub token: Token,
    /// Identifier equality uses canonical names; other tokens compare their source spelling.
    pub name: Option<NameId>,
}
/// Target-sized depth without raising every macro node's alignment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RepetitionDepth([u8; 8]);
impl RepetitionDepth {
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value.to_le_bytes())
    }
    #[must_use]
    pub const fn get(self) -> u64 {
        u64::from_le_bytes(self.0)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expression {
    Count {
        name: NameId,
        depth: RepetitionDepth,
    },
    Ignore(NameId),
    Index(RepetitionDepth),
    Length(RepetitionDepth),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Atom(Atom),
    Crate,
    Doc {
        inner: bool,
        body: Span,
    },
    Group {
        delimiter: Delimiter,
        children: List,
    },
    /// Template operands share the ordinary node and edge arenas.
    Concat(List),
    Expression(Expression),
    EscapedDollar(Token),
    Capture(VariableId),
    Substitute {
        name: NameId,
        dollar: Token,
        identifier: Token,
    },
    Repeat {
        children: List,
        separator: Option<Atom>,
        operator: Repetition,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Node {
    pub span: Span,
    pub kind: Kind,
}
pub struct Variable {
    pub name: NameId,
    pub fragment: Fragment,
    pub repetitions: List,
}
pub struct Rule {
    pub matcher: List,
    pub template: List,
    pub variables: Vec<Variable>,
}
#[derive(Default)]
pub struct Definition {
    pub nodes: Vec<Node>,
    pub edges: Vec<NodeId>,
    pub rules: Vec<Rule>,
}
impl Definition {
    #[must_use]
    pub fn list(&self, list: List) -> &[NodeId] {
        &self.edges[list.start as usize..list.end as usize]
    }
    #[must_use]
    pub fn node(&self, id: NodeId) -> Node {
        self.nodes[id.0 as usize]
    }
}
