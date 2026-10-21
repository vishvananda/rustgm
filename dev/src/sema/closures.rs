//! Typed closure captures and callable contracts; source inference supplies place uses.
pub mod captures;
pub mod imports;
mod source_roots;
pub use source_roots::FormationInput;

use crate::{
    resolve::{model::DefId, unit::NodeKey},
    syntax::ast::List,
    types::{
        RegionId, TypeId,
        closure::{CallKind, ClosureId},
    },
};
use std::collections::HashMap;

/// Source body identity is independent of its enclosing function's declaration.
#[derive(Clone, Copy)]
pub struct Body {
    pub identity: ClosureId,
    pub source: NodeKey,
    pub body: NodeKey,
    pub parameters: List,
    pub signature: TypeId,
    pub owner: Option<DefId>,
    pub universals: u32,
    /// Leading body universals inherited from the immediate enclosing body.
    pub inherited: u32,
    /// Captured bodies have a distinct checked receiver prefix; public inputs stay unchanged.
    pub execution: Option<Execution>,
}
impl Body {
    /// Semantic ownership remains distinct from the syntax root executed by the CFG.
    #[must_use]
    pub const fn semantic_source(&self) -> NodeKey {
        self.source
    }
}
#[derive(Clone, Copy)]
pub struct Execution {
    pub self_ty: TypeId,
    pub signature: TypeId,
    pub universals: u32,
    pub receiver: CallKind,
}
/// The typed consumer that applies implicit dereferences to a source place.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AccessKind {
    Method,
    Invocation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AdjustedAccess {
    /// Exact selected place before capture precision truncation.
    pub path: captures::PathId,
    /// The selected consumer must validate and skip this already-applied prefix.
    pub dereferences: u32,
}
/// Typed capture paths retain their source bindings and physical projections.
pub struct Environment {
    pub plan: captures::Plan,
    /// Types in interned path order, in the checked closure body's lifetime context.
    pub path_types: Vec<TypeId>,
    /// The same paths closed back into their immediate enclosing body's context.
    pub formation_types: Vec<TypeId>,
    /// Stored value/reference types in selected capture order.
    pub field_types: Vec<TypeId>,
    pub accesses: HashMap<NodeKey, captures::PathId>,
    pub adjusted_accesses: HashMap<(NodeKey, AccessKind), AdjustedAccess>,
    pub nested_accesses: HashMap<(NodeKey, captures::PathId), captures::PathId>,
    /// Explicit non-reading uses still require initialized parent places.
    pub mentions: Vec<captures::PathId>,
    pub binding_mutability: HashMap<DefId, bool>,
}
#[derive(Default)]
pub struct Bodies {
    pub(crate) inputs: HashMap<NodeKey, Vec<RegionId>>,
    pub(crate) values: Vec<Body>,
    pub(crate) sources: HashMap<NodeKey, usize>,
    identities: Vec<u32>,
    imports: Option<imports::Imports>,
    pub(crate) environments: HashMap<ClosureId, Environment>,
}
impl Bodies {
    pub(crate) fn lexical_bindings(
        &mut self,
        resolution: &crate::resolve::model::Resolution,
        scope: crate::resolve::model::ScopeId,
    ) -> Result<&[DefId], &'static str> {
        if self.imports.is_none() {
            self.imports = Some(imports::Imports::collect(resolution)?);
        }
        self.imports
            .as_ref()
            .map(|imports| imports.bindings(scope))
            .ok_or("missing closure import index")
    }

    #[must_use]
    pub fn environment(&self, identity: ClosureId) -> Option<&Environment> {
        self.environments.get(&identity)
    }
    #[must_use]
    pub fn identity(&self, identity: ClosureId) -> Option<&Body> {
        let index = self.identities.get(identity.0 as usize)?.checked_sub(1)?;
        self.values.get(index as usize)
    }
    pub(crate) fn insert(&mut self, body: Body) -> Result<(), &'static str> {
        let identity = body.identity.0 as usize;
        if self.sources.contains_key(&body.source)
            || self
                .identities
                .get(identity)
                .is_some_and(|&index| index != 0)
        {
            return Err("closure body published twice");
        }
        let index = u32::try_from(self.values.len())
            .ok()
            .and_then(|index| index.checked_add(1))
            .ok_or("too many closure bodies")?;
        self.identities
            .resize(self.identities.len().max(identity + 1), 0);
        self.identities[identity] = index;
        self.sources.insert(body.source, self.values.len());
        self.values.push(body);
        Ok(())
    }
    #[must_use]
    pub fn get(&self, source: NodeKey) -> Option<&Body> {
        self.values.get(*self.sources.get(&source)?)
    }
    #[must_use]
    pub fn all(&self) -> &[Body] {
        &self.values
    }
}
