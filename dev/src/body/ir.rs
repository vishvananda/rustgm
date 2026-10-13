//! Function-owned flat control-flow storage. Values snapshot reads before later effects.
pub use crate::resolve::unit::NodeKey;
pub use crate::sema::members::Selection;
use crate::{
    lex::token::Punctuation,
    resolve::model::DefId,
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProjectionId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GuardId(pub u32);
/// A guard protects tested places shallowly and its selected binding views deeply.
pub struct Guard {
    pub tested_start: u32,
    pub tested_end: u32,
    pub views_start: u32,
    pub views_end: u32,
}
#[derive(Clone, Copy, Debug)]
pub struct ValueList {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Local(LocalId),
    Static(DefId),
    Temporary(ValueId),
    Projection(ProjectionId),
}
#[derive(Clone, Copy, Debug)]
pub enum ProjectionKind {
    /// Referent access preserves the pointer's region and access capability.
    Dereference {
        region: crate::types::RegionId,
        mutable: bool,
    },
    Field(crate::sema::members::Selection),
    Index {
        index: ValueId,
        length: u64,
    },
    /// Pattern selection with a statically checked bound; permits owned element moves.
    ConstantIndex(u64),
    /// A fixed-array pattern's half-open range, with independently tracked overlap.
    Subarray {
        start: u64,
        end: u64,
    },
}
pub struct Projection {
    pub base: Place,
    pub kind: ProjectionKind,
    pub ty: TypeId,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalKind {
    Parameter(u32),
    Binding,
    Merge,
    /// Internal shared view; native storage holds an address, never an owned snapshot.
    Guard(GuardId),
}
pub struct Local {
    /// Lexical endpoint; explicit `StorageEnd` can additionally close synthetic locals.
    pub scope: super::scopes::Id,
    /// Persistent predecessor in the live storage stack at this local's creation.
    pub storage_parent: Option<LocalId>,
    pub ty: TypeId,
    pub kind: LocalKind,
    pub definition: Option<DefId>,
    pub mutable: bool,
    pub source: NodeId,
}
#[derive(Clone, Copy, Debug)]
pub enum Rvalue {
    Unit,
    Aggregate {
        variant: Option<crate::sema::members::VariantId>,
        fields: ValueList,
    },
    Repeat {
        value: ValueId,
    },
    Scalar(ScalarId),
    /// A by-value use; ownership and Copy selection are checked by later stages.
    Read(Place),
    /// Form a reference to a place without reading or moving its referent.
    Borrow {
        place: Place,
    },
    /// Non-moving constructor test; the owned input must be fully initialized.
    IsVariant {
        place: Place,
        variant: crate::sema::members::VariantId,
    },
    /// Prove an impossible match's input is initialized before ending its CFG.
    Validate(Place),
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
/// A value used as a temporary place. Its scope belongs to this use's syntax,
/// which can differ from the expression that originally produced the value.
pub struct Temporary {
    pub lifetime: super::scopes::Lifetime,
    pub value: ValueId,
    pub source: NodeId,
    pub block: BlockId,
    /// Materialization happens immediately before this global statement index.
    pub before: u32,
    /// Number of previously emitted storage exits; orders events at the same point.
    pub exits_before: u32,
}
#[derive(Clone, Copy, Debug)]
pub enum Action {
    /// Start a new guard attempt; previous attempts cannot keep protections live.
    GuardStart(GuardId),
    /// Keep guard protections live on paths that finish evaluating the guard.
    GuardUse(GuardId),
    /// Select a protected binding address after the pattern decision succeeds.
    BindView {
        view: LocalId,
        place: Place,
    },
    /// Form a place once; indexing checks bounds even when its value is discarded.
    Project(ProjectionId),
    /// Re-form a static pattern path relative to an already evaluated scrutinee.
    Select {
        projection: ProjectionId,
        anchor: Place,
    },
    StorageLive(LocalId),
    StorageDead(LocalId),
    /// End a persistent live-stack suffix, inclusively from last back through first.
    StorageEnd {
        first: LocalId,
        last: LocalId,
    },
    Evaluate(ValueId),
    Store {
        place: Place,
        value: ValueId,
    },
}
impl Action {
    #[must_use]
    pub const fn end_storage(first: LocalId, last: LocalId) -> Self {
        if first.0 == last.0 {
            Self::StorageDead(first)
        } else {
            Self::StorageEnd { first, last }
        }
    }
}
pub struct Statement {
    pub source: NodeId,
    pub action: Action,
}
#[derive(Clone, Copy, Debug)]
pub enum Terminator {
    Jump(BlockId),
    /// The imaginary successor participates in ownership checking, never execution.
    FalseEdge {
        real: BlockId,
        imaginary: BlockId,
    },
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
    pub storage: Option<super::storage::Facts>,
    pub promotions: Option<super::promote::Facts>,
    pub temporaries: Vec<Temporary>,
    pub guards: Vec<Guard>,
    /// Proof-only tested places, including places in inactive variants. Never evaluated.
    pub guard_places: Vec<Place>,
    pub projections: Vec<Projection>,
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
/// Allocation-free reverse traversal of a persistent live-storage suffix.
pub struct StorageRange<'a> {
    locals: &'a [Local],
    first: LocalId,
    next: Option<LocalId>,
}
impl Body {
    #[must_use]
    pub fn storage_range(&self, first: LocalId, last: LocalId) -> StorageRange<'_> {
        StorageRange {
            locals: &self.locals,
            first,
            next: Some(last),
        }
    }
}
impl Iterator for StorageRange<'_> {
    type Item = Result<LocalId, &'static str>;
    fn next(&mut self) -> Option<Self::Item> {
        let id = self.next.take()?;
        let Some(local) = self
            .locals
            .get(id.0 as usize)
            .filter(|_| id.0 >= self.first.0)
        else {
            return Some(Err("invalid storage range endpoint"));
        };
        if id != self.first {
            let Some(parent) = local
                .storage_parent
                .filter(|p| p.0 < id.0 && p.0 >= self.first.0)
            else {
                return Some(Err("storage range does not follow live ancestry"));
            };
            self.next = Some(parent);
        }
        Some(Ok(id))
    }
}
pub struct Program {
    pub bodies: Vec<Body>,
    pub region_equalities: Vec<RegionEquality>,
    pub coercions: Vec<crate::sema::model::Coercion>,
    pub outlives: Vec<crate::sema::model::Outlives>,
    pub body_regions: Vec<crate::sema::model::BodyRegion>,
}
