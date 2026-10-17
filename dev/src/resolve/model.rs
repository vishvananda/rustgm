//! Stable name-resolution facts; lexical bindings point directly at definitions.
pub use super::builtins::{BuiltinTrait, Traits as BuiltinTraits};
use super::{names::NameId, package::ModuleId, unit::NodeKey};
use crate::syntax::ast::NodeId;
use std::collections::HashMap;

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
    pub bindings: HashMap<(NameId, Namespace), Binding>,
    pub order: Vec<(NameId, Namespace)>,
}
#[derive(Clone, Debug)]
pub struct Import {
    pub scope: ScopeId,
    pub node: NodeKey,
    pub path: Vec<NameId>,
    pub absolute: bool,
    pub alias: Option<NameId>,
    pub glob: bool,
    pub access: Access,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Definition(DefId),
    Associated(DefId),
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
    pub definitions: HashMap<DefId, crate::expand::stream::ContextId>,
    pub imports: HashMap<usize, crate::expand::stream::ContextId>,
    pub parents: Vec<crate::expand::stream::ContextId>,
}
impl Introductions {
    #[must_use]
    pub fn binding(&self, binding: Binding) -> crate::expand::stream::ContextId {
        use crate::expand::stream::ContextId;
        match binding.origin {
            Origin::Direct => self.definitions.get(&binding.target),
            Origin::Import(index) | Origin::Glob(index) => self.imports.get(&(index as usize)),
        }
        .copied()
        .unwrap_or(ContextId(0))
    }
    #[must_use]
    pub fn ancestor(
        &self,
        ancestor: crate::expand::stream::ContextId,
        mut context: crate::expand::stream::ContextId,
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
/// Associated binding names are canonical, while their member is selected from typed bounds.
#[derive(Clone, Copy)]
pub struct AssociatedBinding {
    pub name: NameId,
    pub arguments: Option<crate::syntax::ast::List>,
}

#[derive(Default)]
pub struct Resolution {
    pub builtin_traits: BuiltinTraits,
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
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
    pub associated_paths: HashMap<NodeKey, AssociatedPath>,
    pub qualified_paths: HashMap<NodeKey, QualifiedPath>,
    pub associated_bindings: HashMap<NodeKey, AssociatedBinding>,
    pub method_calls: HashMap<NodeKey, MethodCall>,
    pub imports: Vec<Import>,
    pub module_scopes: Vec<ScopeId>,
    pub node_scopes: HashMap<(ModuleId, NodeId), ScopeId>,
    pub item_definitions: HashMap<(ModuleId, NodeId, Namespace), DefId>,
    pub primitives: HashMap<NameId, DefId>,
    pub expansions: Vec<(ScopeId, NodeKey)>,
}
