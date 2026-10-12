//! Function-owned flat control-flow storage. Values snapshot reads before later effects.
use crate::{
    lex::token::Punctuation,
    resolve::{model::DefId, unit::NodeKey},
    sema::model::{Cast, InitializerKind, RegionEquality, Scalar},
    syntax::ast::{NodeId, Unary},
    types::TypeId,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LocalId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ValueId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScalarId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);
#[derive(Clone, Copy, Debug)]
pub struct ValueList {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Local(LocalId),
    Static(DefId),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalKind {
    Parameter(u32),
    Binding,
    Merge,
}
pub struct Local {
    pub ty: TypeId,
    pub kind: LocalKind,
    pub definition: Option<DefId>,
    pub mutable: bool,
    pub source: NodeId,
}
#[derive(Clone, Copy, Debug)]
pub enum Rvalue {
    Unit,
    Scalar(ScalarId),
    /// A by-value use; ownership and Copy selection are checked by later stages.
    Read(Place),
    Function(DefId),
    Constant(DefId),
    Reify(ValueId),
    Constrain(ValueId),
    Unary {
        operator: Unary,
        operand: ValueId,
        literal: bool,
    },
    Binary {
        operator: Punctuation,
        left: ValueId,
        right: ValueId,
    },
    Cast {
        kind: Cast,
        operand: ValueId,
    },
    Call {
        signature: TypeId,
        callee: ValueId,
        arguments: ValueList,
    },
}
pub struct Value {
    pub ty: TypeId,
    pub kind: Rvalue,
}
#[derive(Clone, Copy, Debug)]
pub enum Action {
    StorageLive(LocalId),
    StorageDead(LocalId),
    Evaluate(ValueId),
    Store { place: Place, value: ValueId },
}
pub struct Statement {
    pub source: NodeId,
    pub action: Action,
}
#[derive(Clone, Copy, Debug)]
pub enum Terminator {
    Jump(BlockId),
    Branch {
        condition: ValueId,
        yes: BlockId,
        no: BlockId,
    },
    Return(ValueId),
    Unreachable,
}
pub struct Block {
    pub start: u32,
    pub end: u32,
    pub terminator: Terminator,
    pub source: NodeId,
}
#[derive(Clone, Copy)]
pub enum Owner {
    Function {
        definition: DefId,
        signature: TypeId,
    },
    Initializer {
        definition: DefId,
        kind: InitializerKind,
    },
}
pub struct Body {
    pub owner: Owner,
    pub source: NodeKey,
    pub result: TypeId,
    pub locals: Vec<Local>,
    pub values: Vec<Value>,
    pub scalars: Vec<Scalar>,
    pub arguments: Vec<ValueId>,
    pub statements: Vec<Statement>,
    pub blocks: Vec<Block>,
}
impl Body {
    #[must_use]
    pub fn statements(&self, block: &Block) -> &[Statement] {
        &self.statements[block.start as usize..block.end as usize]
    }
    #[must_use]
    pub fn arguments(&self, list: ValueList) -> &[ValueId] {
        &self.arguments[list.start as usize..list.end as usize]
    }
}
pub struct Program {
    pub bodies: Vec<Body>,
    pub region_equalities: Vec<RegionEquality>,
}
