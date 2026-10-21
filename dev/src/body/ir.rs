//! Function-owned flat control-flow storage. Values snapshot reads before later effects.
pub use crate::resolve::{model::DefId, unit::NodeKey};
pub use crate::sema::members::{Selection, VariantId};
use crate::{
    lex::token::Punctuation,
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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Place {
    Local(LocalId),
    Static(DefId),
    Temporary(ValueId),
    Projection(ProjectionId),
}
#[derive(Clone, Copy, Debug)]
pub enum ProjectionKind {
    /// Preserve source binding authority after dereferencing a stored capture.
    /// This boundary has no physical field, address offset or owned move path.
    CaptureBoundary {
        mutable: bool,
    },
    /// Referent access preserves the pointer's region and access capability.
    Dereference {
        region: crate::types::RegionId,
        mutable: bool,
    },
    /// Raw access has no reference region, loan ancestry or owned move path.
    RawDereference {
        mutable: bool,
    },
    Field(crate::sema::members::Selection),
    /// Different union fields overlap before any descendant disjointness test.
    UnionField(crate::sema::members::Selection),
    Index {
        index: ValueId,
        length: u64,
    },
    IndexParameter {
        index: ValueId,
        length: crate::types::arguments::ConstId,
    },
    /// Bounds come from the slice place's retained pointer metadata.
    SliceIndex {
        index: ValueId,
    },
    /// Pattern selection with a statically checked bound; permits owned element moves.
    ConstantIndex(u64),
    /// One-based suffix selection, justified by the enclosing pattern's length test.
    FromEnd {
        offset: u64,
        minimum: u64,
    },
    /// Dynamic rest selection excludes a fixed prefix and suffix.
    Subslice {
        prefix: u64,
        suffix: u64,
    },
    /// A fixed-array pattern's half-open range, with independently tracked overlap.
    Subarray {
        start: u64,
        end: u64,
    },
}
impl ProjectionKind {
    /// Both forms evaluate pointer storage before selecting a referent place.
    #[must_use]
    pub const fn is_dereference(self) -> bool {
        matches!(self, Self::Dereference { .. } | Self::RawDereference { .. })
    }
}
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RepeatMode {
    /// Evaluate an ordinary operand once, even when the array is empty.
    Once,
    /// Instantiate a selected constant path independently for each element.
    Constant,
}
#[derive(Clone, Copy, Debug)]
pub enum Rvalue {
    Unit,
    Aggregate {
        variant: Option<VariantId>,
        fields: ValueList,
    },
    /// Initialize only the selected field's bytes, leaving other storage untouched.
    Union {
        field: Selection,
        value: ValueId,
    },
    Repeat {
        value: ValueId,
        mode: RepeatMode,
    },
    Scalar(ScalarId),
    Text(crate::sema::text::Id),
    /// Inspect metadata without reading or moving the elements.
    Length(Place),
    /// Compare the exact bytes of a str place without moving it.
    TextEquals {
        place: Place,
        text: crate::sema::text::Id,
    },
    /// A by-value use; ownership and Copy selection are checked by later stages.
    Read(Place),
    /// Form a raw pointer; check access now, without creating a reference loan.
    Address(Place),
    /// Form a reference to a place without reading or moving its referent.
    Borrow {
        place: Place,
        kind: BorrowKind,
    },
    /// Non-moving constructor test; the owned input must be fully initialized.
    IsVariant {
        place: Place,
        variant: VariantId,
    },
    /// Prove an impossible match's input is initialized before ending its CFG.
    Validate(Place),
    Function(DefId),
    Constant(DefId),
    AssociatedConstant(crate::sema::values::Instance),
    /// A typed anonymous body, evaluated independently of runtime branch selection.
    InlineConstant(NodeKey),
    /// A generic inline body retains its complete enclosing specialization.
    InlineInstance(crate::sema::inline_constants::InlineInstance),
    /// A canonical scalar const argument; rigid parameters remain unevaluated.
    ConstArgument(crate::types::arguments::ConstId),
    Reify(ValueId),
    Constrain(ValueId),
    /// A selected structural unsizing retains every terminal proof input.
    Unsize {
        value: ValueId,
        conversion: crate::types::unsizing::Conversion<TypeId>,
    },
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
    /// A checked call operator; operand zero owns or borrows the selected receiver.
    Invoke {
        kind: crate::support::call_traits::CallTrait,
        signature: TypeId,
        arguments: ValueList,
    },
    Call {
        signature: TypeId,
        callee: ValueId,
        arguments: ValueList,
    },
    /// A canonical language operation, with independent unsafe and const permissions.
    Intrinsic {
        intrinsic: crate::support::intrinsics::Intrinsic,
        instance: crate::types::instance::Instance,
        arguments: ValueList,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BorrowKind {
    Immediate,
    TwoPhase,
    /// Capture-only exclusive formation; source binding replacement stays separate.
    UniqueImmutable,
}
#[derive(Clone, Copy)]
pub struct Value {
    pub ty: TypeId,
    pub kind: Rvalue,
}
/// A value used as a temporary place. Its scope belongs to this use's syntax,
/// which can differ from the expression that originally produced the value.
#[derive(Clone, Copy)]
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
    /// Closure creation requires initialized wildcard inputs without accessing them.
    CheckInitialized(Place),
    /// A wildcard mentions a place's type without reading, moving or borrowing it.
    Mention(Place),
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
#[derive(Clone, Copy)]
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
#[derive(Clone, Copy)]
pub struct Block {
    pub start: u32,
    pub end: u32,
    pub terminator: Terminator,
    pub source: NodeId,
}
#[derive(Clone, Copy)]
pub enum Owner {
    ClosureAdapter {
        adapter: crate::types::closure::Adapter,
        signature: TypeId,
    },
    Closure {
        identity: crate::types::closure::ClosureId,
        signature: TypeId,
        /// Checked hidden receiver signature, separate from the public callable.
        execution: Option<TypeId>,
    },
    /// A declaration-time constant requiring lifetime/loan proof, never a native function.
    AnonymousConstant,
    AnonymousInstance(crate::sema::inline_constants::InlineInstance),
    Function {
        definition: DefId,
        signature: TypeId,
    },
    Initializer {
        definition: DefId,
        kind: InitializerKind,
    },
}
/// Declaration identity is separate from shared diagnostic provenance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BodyKey {
    ClosureAdapter(crate::types::closure::Adapter),
    Declaration(DefId),
    Closure(crate::types::closure::ClosureId),
    Anonymous(NodeKey),
    AnonymousInstance(crate::sema::inline_constants::InlineInstance),
}
#[derive(Clone)]
pub struct Body {
    /// Canonical types of static places actually referenced by this CFG.
    pub static_types: Option<std::sync::Arc<std::collections::HashMap<DefId, TypeId>>>,
    /// Immutable declaration permissions, shared by checked bodies and instances.
    pub mutable_statics: Option<std::sync::Arc<std::collections::HashSet<DefId>>>,
    /// Constant eligibility applies independently of whether execution is demanded.
    pub const_context: bool,
    /// Concrete specialization identity; diagnostics keep the original source owner.
    pub instance: Option<crate::types::instance::Instance>,
    /// Destructible rvalues, including values never materialized as borrowed places.
    pub owned: Option<Box<super::drops::values::Facts>>,
    pub storage: Option<super::storage::Facts>,
    pub promotions: Option<super::promote::Facts>,
    /// Lifetime-equivalent callable identities selected only after a complete body proof.
    pub resolved_callables: Option<Box<ResolvedCallables>>,
    /// Selected checked environment entries; created before the ordinary body proof.
    pub closure_calls: Option<Box<super::invocation_entries::Entries>>,
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
#[derive(Clone, Default)]
pub struct ResolvedCallables {
    targets: std::collections::HashMap<
        crate::types::instance::Instance,
        crate::types::instance::Instance,
    >,
}
impl ResolvedCallables {
    pub(crate) fn insert(
        &mut self,
        original: crate::types::instance::Instance,
        target: crate::types::instance::Instance,
    ) {
        self.targets.insert(original, target);
    }
}
impl Body {
    /// Owned payload only: shared static metadata and canonical arena values
    /// remain charged to their original owners, never to each cloned body.
    pub(crate) fn allocation_bytes(&self) -> usize {
        use crate::support::retained::{map, sum, vector};
        sum([
            self.closure_calls
                .as_ref()
                .map_or(0, |calls| calls.retained_bytes()),
            vector(&self.temporaries),
            vector(&self.guards),
            vector(&self.guard_places),
            vector(&self.projections),
            vector(&self.locals),
            vector(&self.values),
            vector(&self.scalars),
            vector(&self.arguments),
            vector(&self.statements),
            vector(&self.blocks),
            self.storage.as_ref().map_or(0, |facts| {
                sum([vector(&facts.scopes), vector(&facts.exits)])
            }),
            self.owned.as_ref().map_or(0, |facts| {
                size_of::<super::drops::values::Facts>().saturating_add(facts.allocation_bytes())
            }),
            self.promotions
                .as_ref()
                .map_or(0, super::promote::Facts::allocation_bytes),
            self.resolved_callables.as_ref().map_or(0, |facts| {
                size_of::<ResolvedCallables>().saturating_add(map(&facts.targets))
            }),
        ])
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(self.allocation_bytes())
    }
    pub(crate) fn callable_allocation_bytes(&self) -> usize {
        self.resolved_callables
            .as_ref()
            .map_or(0, |facts| crate::support::retained::map(&facts.targets))
    }
    /// Read place types from retained CFG facts, including external static storage.
    #[must_use]
    pub fn place_type(&self, place: Place) -> Option<TypeId> {
        match place {
            Place::Local(id) => self.locals.get(id.0 as usize).map(|l| l.ty),
            Place::Temporary(id) => self.values.get(id.0 as usize).map(|v| v.ty),
            Place::Projection(id) => self.projections.get(id.0 as usize).map(|p| p.ty),
            Place::Static(id) => self.static_types.as_ref()?.get(&id).copied(),
        }
    }
    #[must_use]
    pub const fn key(&self) -> BodyKey {
        match self.owner {
            Owner::ClosureAdapter { adapter, .. } => BodyKey::ClosureAdapter(adapter),
            Owner::Function { definition, .. } | Owner::Initializer { definition, .. } => {
                BodyKey::Declaration(definition)
            }
            Owner::AnonymousConstant => BodyKey::Anonymous(self.source),
            Owner::AnonymousInstance(instance) => BodyKey::AnonymousInstance(instance),
            Owner::Closure { identity, .. } => BodyKey::Closure(identity),
        }
    }
    pub(crate) fn callable_targets(
        &self,
    ) -> std::collections::HashMap<crate::types::instance::Instance, crate::types::instance::Instance>
    {
        self.resolved_callables
            .as_ref()
            .map_or_else(std::collections::HashMap::new, |resolved| {
                resolved.targets.clone()
            })
    }
    #[must_use]
    pub fn callable_target(
        &self,
        original: crate::types::instance::Instance,
    ) -> crate::types::instance::Instance {
        self.resolved_callables
            .as_ref()
            .and_then(|resolved| resolved.targets.get(&original))
            .copied()
            .unwrap_or(original)
    }
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
impl<'a> StorageRange<'a> {
    pub(crate) const fn new(locals: &'a [Local], first: LocalId, last: LocalId) -> Self {
        Self {
            locals,
            first,
            next: Some(last),
        }
    }
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
    pub drops: Option<super::drops::Pending>,
    pub bodies: Vec<Body>,
    pub region_equalities: Vec<RegionEquality>,
    pub coercions: Vec<crate::sema::model::Coercion>,
    pub outlives: Vec<crate::sema::model::Outlives>,
    pub body_regions: Vec<crate::sema::model::BodyRegion>,
}
