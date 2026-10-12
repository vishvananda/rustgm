//! Stable name-resolution facts; lexical bindings point directly at definitions.
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
pub struct Scope {
    pub parent: Option<ScopeId>,
    pub module: ModuleId,
    pub barrier: bool,
    pub associated: bool,
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
#[derive(Default)]
pub struct Resolution {
    pub introductions: Introductions,
    pub textual_imports: HashMap<usize, Binding>,
    pub crate_root: Option<DefId>,
    pub scopes: Vec<Scope>,
    pub definitions: Vec<Definition>,
    pub references: Vec<Reference>,
    pub imports: Vec<Import>,
    pub module_scopes: Vec<ScopeId>,
    pub node_scopes: HashMap<(ModuleId, NodeId), ScopeId>,
    pub item_definitions: HashMap<(ModuleId, NodeId, Namespace), DefId>,
    pub primitives: HashMap<NameId, DefId>,
    pub expansions: Vec<(ScopeId, NodeKey)>,
}
