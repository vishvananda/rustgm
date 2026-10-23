//! Artifact-local rows contain requests and obligations, never frontend permissions.
use crate::support::{
    crates::{
        CrateIdentity, ExternalDef, Scalar,
        owner_catalog::{BodyRef, DeclarationRef},
    },
    edition::Edition,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TypeRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BinderRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ApplicationRef(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Declaration {
    Function {
        identity: ExternalDef,
        public: bool,
    },
    TypeParameter {
        owner: DeclarationRef,
        ordinal: u32,
        name: String,
        sized: bool,
    },
    /// A real named input binding, distinct from an unnamed incoming value slot.
    ValueParameter {
        body: BodyRef,
        ordinal: u32,
        name: String,
    },
    Local {
        body: BodyRef,
        ordinal: u32,
        name: String,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinderKind {
    OriginalFunction,
    PublicFunction,
    Clause,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binder {
    pub owner: DeclarationRef,
    pub parent: Option<BinderRef>,
    pub kind: BinderKind,
    pub count: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Convention {
    Rust,
    RustCall,
    C,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signature {
    pub binder: BinderRef,
    pub parameters: Range,
    pub result: TypeRef,
    pub unsafe_: bool,
    pub convention: Convention,
    pub variadic: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Scalar(Scalar),
    Parameter(DeclarationRef),
    Function(Signature),
    FunctionItem(ApplicationRef),
    ApplicationSignature(ApplicationRef),
}
/// Complete static applications are data, not selected-call permissions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Application {
    pub target: DeclarationRef,
    pub arguments: Range,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Calls {
    pub header_types: u32,
    pub applications: Vec<Application>,
    pub arguments: Vec<TypeRef>,
}
/// A source-defined Copy contract never becomes a spelling or legacy builtin bit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopyIdentity {
    LegacyBuiltin,
    Source {
        owner: CrateIdentity,
        declaration: DeclarationRef,
    },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Bound {
    Sized,
    Copy(CopyIdentity),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clause {
    pub owner: DeclarationRef,
    pub site: u32,
    pub subject: TypeRef,
    pub quantifier: Option<BinderRef>,
    pub bound: Bound,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Function {
    pub declaration: DeclarationRef,
    pub body: BodyRef,
    pub original: TypeRef,
    pub callable: TypeRef,
    pub original_frame: BinderRef,
    pub callable_frame: BinderRef,
    /// Complete declared order, including unused slots.
    pub parameters: Range,
    pub clauses: Range,
    pub declared_clauses: u32,
    /// Declared const eligibility still requires original and instance checks.
    pub constant: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LocalKind {
    Parameter(u32),
    Binding,
    Merge,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Local {
    pub scope: u32,
    pub storage_parent: Option<u32>,
    pub ty: TypeRef,
    pub kind: LocalKind,
    pub definition: Option<DeclarationRef>,
    pub mutable: bool,
    pub site: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    Unit,
    Read(u32),
    Constrain(u32),
    Function(ApplicationRef),
    Call {
        signature: TypeRef,
        callee: u32,
        arguments: Range,
        site: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Value {
    pub ty: TypeRef,
    pub kind: ValueKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    StorageLive(u32),
    StorageDead(u32),
    StorageEnd { first: u32, last: u32 },
    Evaluate(u32),
    Store { local: u32, value: u32 },
    CheckInitialized(u32),
    Mention(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Statement {
    pub site: u32,
    pub action: Action,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Terminator {
    Return(u32),
    Jump(u32),
    Unreachable,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    pub statements: Range,
    pub site: u32,
    pub terminator: Terminator,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeKind {
    Body,
    Block,
    Statement,
    Tail,
    Condition,
    IfThen,
    Iteration,
    Arm,
    Guard,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Scope {
    pub parent: Option<u32>,
    pub kind: ScopeKind,
    pub site: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Location {
    At { block: u32, before: u32 },
    Edge { block: u32, target: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lifetime {
    Scope(u32),
    Static,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Temporary {
    pub lifetime: Lifetime,
    pub value: u32,
    pub site: u32,
    pub block: u32,
    pub before: u32,
    pub exits_before: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum End {
    Scopes { inner: u32, outer: Option<u32> },
    Temporaries(Range),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exit {
    pub location: Location,
    pub end: End,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ParameterBinding {
    pub declaration: DeclarationRef,
    pub input: u32,
    pub local: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Original {
    pub body: BodyRef,
    pub declaration: DeclarationRef,
    pub result: TypeRef,
    pub sites: u32,
    pub locals: Vec<Local>,
    pub parameter_bindings: Vec<ParameterBinding>,
    pub values: Vec<Value>,
    pub statements: Vec<Statement>,
    pub blocks: Vec<Block>,
    pub scopes: Vec<Scope>,
    pub exits: Vec<Exit>,
    pub temporaries: Vec<Temporary>,
    pub arguments: Vec<u32>,
}
/// Complete raw input of the first operation family. Unsupported source records
/// must reject before construction; no omitted projection/guard/temporary table
/// is inferred to be empty by an exporter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rows {
    pub edition: Edition,
    pub declarations: Vec<Declaration>,
    pub declaration_sites: Vec<u32>,
    pub binders: Vec<Binder>,
    pub types: Vec<Type>,
    pub roots: Vec<TypeRef>,
    pub parameters: Vec<DeclarationRef>,
    pub clauses: Vec<Clause>,
    pub functions: Vec<Function>,
    pub originals: Vec<Original>,
    pub calls: Option<Box<Calls>>,
}
