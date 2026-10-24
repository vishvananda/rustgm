//! Stable name-resolution facts; lexical bindings point directly at definitions.
pub use super::builtins::{
    BuiltinIntrinsic, BuiltinTrait, Callable as BuiltinCallable,
    CloneImplementation as BuiltinCloneImplementation, Cloning as BuiltinCloning,
    CollectionDeclaration as BuiltinCollectionDeclaration, Collections as BuiltinCollections,
    ComparisonMembers as BuiltinComparisonMembers, ComparisonResult, Equality as BuiltinEquality,
    Intrinsics, Iteration as BuiltinIteration, IterationBlanket as BuiltinIterationBlanket,
    IterationDeclaration as BuiltinIterationDeclaration, Language as BuiltinLanguage,
    LanguageOutput as BuiltinLanguageOutput, LibraryNominal, LibraryTypes,
    MutableCollections as BuiltinMutableCollections,
    RangeImplementation as BuiltinRangeImplementation, Ranges as BuiltinRanges,
    SliceLength as LibrarySliceLength, Traits as BuiltinTraits,
};
pub use super::table::BindingTable;
use super::{
    names::NameId,
    package::ModuleId,
    unit::{FileId, NodeKey},
};
use crate::syntax::ast::NodeId;
use crate::{expand::marks::ItemContext, support::hash::HashMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ScopeId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DefId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Namespace {
    Type,
    Value,
    Macro,
    Lifetime,
    Label,
}
impl Namespace {
    pub const IMPORTABLE: [Self; 3] = [Self::Type, Self::Value, Self::Macro];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Type => "type",
            Self::Value => "value",
            Self::Macro => "macro",
            Self::Lifetime => "lifetime",
            Self::Label => "label",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Public,
    Within(ModuleId),
    ExternalPrivate,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefKind {
    Primitive,
    BuiltinMacro,
    BuiltinTrait,
    Module,
    Function,
    Struct,
    Constructor,
    Enum,
    Variant,
    Trait,
    TypeAlias,
    Const,
    Static,
    Generic,
    Local,
    Parameter,
    SelfType,
    Macro,
    ExternCrate,
    Label,
    Lifetime,
}
impl DefKind {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Primitive => "primitive",
            Self::BuiltinMacro => "builtin-macro",
            Self::BuiltinTrait => "builtin-trait",
            Self::Module => "module",
            Self::Function => "function",
            Self::Struct => "struct",
            Self::Constructor => "constructor",
            Self::Enum => "enum",
            Self::Variant => "variant",
            Self::Trait => "trait",
            Self::TypeAlias => "alias",
            Self::Const => "const",
            Self::Static => "static",
            Self::Generic => "generic",
            Self::Local => "local",
            Self::Parameter => "parameter",
            Self::SelfType => "self-type",
            Self::Macro => "macro",
            Self::ExternCrate => "extern-crate",
            Self::Label => "label",
            Self::Lifetime => "lifetime",
        }
    }
    #[must_use]
    pub const fn is_local(self) -> bool {
        matches!(
            self,
            Self::Local
                | Self::Parameter
                | Self::Generic
                | Self::Lifetime
                | Self::Label
                | Self::SelfType
        )
    }
}
pub struct Definition {
    pub name: NameId,
    pub namespace: Namespace,
    pub kind: DefKind,
    pub scope: ScopeId,
    pub node: Option<NodeKey>,
    pub access: Access,
    pub members: Option<ScopeId>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    Direct,
    Import(u32),
    Glob(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Binding {
    pub target: DefId,
    pub access: Access,
    pub origin: Origin,
    pub ambiguous: bool,
}
impl Binding {
    /// Variants and associated type members do not shadow enclosing lexical
    /// names. Their qualified member lookup uses the scope directly.
    pub(crate) fn is_lexical(self, definitions: &[Definition], associated: bool) -> bool {
        self.origin != Origin::Direct
            || (definitions[self.target.0 as usize].kind != DefKind::Variant
                && !(associated
                    && matches!(
                        definitions[self.target.0 as usize].kind,
                        DefKind::TypeAlias | DefKind::Const | DefKind::Function
                    )))
    }
}
pub struct Scope {
    pub parent: Option<ScopeId>,
    pub module: ModuleId,
    pub barrier: bool,
    pub associated: bool,
    /// Direct trait members inherit the trait's access; nested items do not.
    pub trait_access: Option<Access>,
    pub nominal_self: Option<DefId>,
    pub label_barrier: bool,
}
#[derive(Clone, Debug)]
pub struct ImportContexts {
    pub segments: Box<[ItemContext]>,
    pub glob: ItemContext,
}
#[derive(Clone, Debug)]
pub struct Import {
    pub scope: ScopeId,
    pub node: NodeKey,
    /// Import retries share the immutable spelling sequence instead of copying it.
    pub path: std::sync::Arc<[NameId]>,
    /// Absent for root-only paths/stars; retries share the cold contextual facts.
    pub contexts: Option<std::sync::Arc<ImportContexts>>,
    pub absolute: bool,
    pub alias: Option<(NameId, ItemContext)>,
    pub glob: bool,
    pub access: Access,
}
impl Import {
    pub(crate) fn segment_context(&self, index: usize) -> ItemContext {
        self.contexts
            .as_ref()
            .map_or(ItemContext::ROOT, |contexts| contexts.segments[index])
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Definition(DefId),
    Associated(DefId),
    /// A value-form `Self` selects its constructor after the impl target is typed.
    SelfConstructor(DefId),
    /// A nested item may use a closed Self constructor, but cannot capture parameters.
    InheritedSelfConstructor(DefId),
    Expansion(ScopeId),
}
/// The namespace-resolved prefix and canonical names of the remaining segments.
/// Semantic member selection consumes these facts without repeating path lookup.
pub struct AssociatedPath {
    pub owner: DefId,
    pub namespace: Namespace,
    pub scope: ScopeId,
    pub prefix: crate::syntax::ast::NodeId,
    pub suffix: Vec<(crate::syntax::ast::NodeId, NameId)>,
}
/// Explicit type qualification keeps canonical suffix names while type analysis
/// elaborates the receiver and optional trait prefix.
pub struct QualifiedPath {
    pub namespace: Namespace,
    pub scope: ScopeId,
    pub ty: NodeId,
    pub trait_path: Option<NodeId>,
    pub suffix: Vec<(NodeId, NameId)>,
}
/// A method spelling and lexical search site, independent of receiver typing.
/// Generic arguments remain in the retained syntax until a method is selected.
#[derive(Clone, Copy)]
pub struct MethodCall {
    pub scope: ScopeId,
    pub receiver: NodeId,
    pub name: NameId,
    pub method: Option<NodeId>,
    pub arguments: crate::syntax::ast::List,
}
pub struct Reference {
    pub scope: ScopeId,
    pub node: NodeKey,
    pub namespace: Namespace,
    pub target: Target,
}
/// Expansion-only binding provenance; ordinary resolution leaves these tables empty.
#[derive(Default)]
pub struct Introductions {
    pub definitions: HashMap<DefId, crate::expand::marks::ExpansionId>,
    pub imports: HashMap<usize, crate::expand::marks::ExpansionId>,
    pub parents: Vec<crate::expand::marks::ExpansionId>,
}
impl Introductions {
    #[must_use]
    pub fn binding(&self, binding: Binding) -> crate::expand::marks::ExpansionId {
        use crate::expand::marks::ExpansionId;
        match binding.origin {
            Origin::Direct => self.definitions.get(&binding.target),
            Origin::Import(index) | Origin::Glob(index) => self.imports.get(&(index as usize)),
        }
        .copied()
        .unwrap_or(ExpansionId(0))
    }
    #[must_use]
    pub fn ancestor(
        &self,
        ancestor: crate::expand::marks::ExpansionId,
        mut context: crate::expand::marks::ExpansionId,
    ) -> bool {
        for _ in 0..=self.parents.len() {
            if context == ancestor {
                return true;
            }
            let Some(index) = context.0.checked_sub(1) else {
                return false;
            };
            let Some(parent) = self.parents.get(index as usize) else {
                return false;
            };
            context = *parent;
        }
        false
    }
}
/// Lexical scopes introduced at syntax nodes. Each module's nodes live in its
/// file, so one dense slot per file node answers ordinary queries; a file
/// instantiated by several modules keeps the other modules' scopes sparse.
#[derive(Default)]
pub struct NodeScopes {
    /// The file owning each module's nodes, recorded by its first insertion.
    files: Vec<Option<FileId>>,
    slots: Vec<Vec<NodeScope>>,
    overflow: HashMap<(ModuleId, NodeId), ScopeId>,
    /// Distinct keys in first-insertion order.
    keys: Vec<(ModuleId, NodeId)>,
}
#[derive(Clone, Copy)]
struct NodeScope {
    module: ModuleId,
    scope: ScopeId,
}
const VACANT: NodeScope = NodeScope {
    module: ModuleId(u32::MAX),
    scope: ScopeId(u32::MAX),
};
impl NodeScopes {
    #[must_use]
    pub fn get(&self, key: &(ModuleId, NodeId)) -> Option<&ScopeId> {
        let (module, node) = *key;
        if let Some(Some(file)) = self.files.get(module.0 as usize)
            && let Some(slot) = self.slots[file.0 as usize].get(node.get() as usize)
            && slot.module == module
        {
            return Some(&slot.scope);
        }
        if self.overflow.is_empty() {
            return None;
        }
        self.overflow.get(key)
    }
    /// Record `scope` for a node of `module`, whose syntax belongs to `file`.
    pub fn insert(&mut self, module: ModuleId, file: FileId, node: NodeId, scope: ScopeId) {
        let index = module.0 as usize;
        if index >= self.files.len() {
            self.files.resize(index + 1, None);
        }
        let owner = *self.files[index].get_or_insert(file);
        if file.0 as usize >= self.slots.len() {
            self.slots.resize_with(file.0 as usize + 1, Vec::new);
        }
        let slots = &mut self.slots[file.0 as usize];
        if owner == file {
            if node.get() as usize >= slots.len() {
                // Scopes spread over the whole file: grow geometrically.
                slots.resize(
                    (node.get() as usize + 1).next_power_of_two().max(64),
                    VACANT,
                );
            }
            let slot = &mut slots[node.get() as usize];
            if slot.module == module {
                slot.scope = scope;
                return;
            }
            if slot.module == VACANT.module {
                *slot = NodeScope { module, scope };
                self.keys.push((module, node));
                return;
            }
        }
        if self.overflow.insert((module, node), scope).is_none() {
            self.keys.push((module, node));
        }
    }
    pub fn keys(&self) -> impl Iterator<Item = &(ModuleId, NodeId)> {
        self.keys.iter()
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.keys.len()
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}
/// Associated binding names are canonical, while their member is selected from typed bounds.
#[derive(Clone, Copy)]
pub struct AssociatedBinding {
    pub name: NameId,
    pub arguments: Option<crate::syntax::ast::List>,
}

#[derive(Default)]
pub struct Resolution {
    pub source_bindings: Option<Box<super::bindings::Reservations>>,
    /// Source-free external identities and their explicit local remapping.
    pub crates: Option<Box<super::crates::Imported>>,
    /// Actual external-block ownership, retained through expansion by declaration identity.
    pub foreign_owners: HashMap<DefId, NodeKey>,
    pub foreign_blocks: crate::support::hash::HashSet<NodeKey>,
    /// Unnameable `impl Trait` input parameters, grouped by their owning function.
    pub anonymous_parameters: HashMap<NodeKey, Vec<NodeKey>>,
    pub builtin_traits: BuiltinTraits,
    pub library_types: LibraryTypes,
    pub intrinsics: Intrinsics,
    /// Language-owned identity, distinct from any user declaration named `Drop`.
    pub drop_trait: Option<DefId>,
    /// Supported external crate metadata, separate from the crate's lexical root.
    pub extern_scope: Option<ScopeId>,
    /// Alternative occurrences of one or-pattern binding share its definition.
    pub pattern_aliases: Vec<(NodeKey, DefId)>,
    /// Field restrictions are resolved once, before semantic field selection.
    pub field_access: HashMap<NodeKey, Access>,
    pub macro_prelude: HashMap<NameId, DefId>,
    pub introductions: Introductions,
    pub textual_imports: HashMap<usize, Binding>,
    /// Anonymous imports have no lexical binding. Retain their final type target
    /// by import identity for trait lookup, including its scope and hygiene.
    pub anonymous_types: HashMap<usize, Binding>,
    pub crate_root: Option<DefId>,
    pub scopes: Vec<Scope>,
    /// Every scope's bindings, in one table keyed by scope, name and namespace.
    pub bindings: BindingTable,
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
    pub associated_paths: HashMap<NodeKey, AssociatedPath>,
    pub qualified_paths: HashMap<NodeKey, QualifiedPath>,
    pub associated_bindings: HashMap<NodeKey, AssociatedBinding>,
    pub method_calls: HashMap<NodeKey, MethodCall>,
    pub imports: Vec<Import>,
    pub module_scopes: Vec<ScopeId>,
    pub node_scopes: NodeScopes,
    pub item_definitions: HashMap<(ModuleId, NodeId, Namespace), DefId>,
    pub primitives: HashMap<NameId, DefId>,
    pub expansions: Vec<(ScopeId, NodeKey)>,
}
