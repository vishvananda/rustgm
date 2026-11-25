//! Typed closure captures and callable contracts; source inference supplies place uses.
pub(crate) mod actual;
pub mod captures;
pub(crate) mod condition_slots;
mod execution_requirements;
pub mod imports;
pub(crate) mod matching;
pub(crate) mod qualification;
mod source_roots;
pub use source_roots::FormationInput;

use crate::support::hash::HashMap;
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    syntax::ast::List,
    types::{
        RegionId, TypeId,
        closure::{CallKind, ClosureId},
    },
};

/// Source body identity is independent of its enclosing function's declaration.
#[derive(Clone, Copy)]
pub struct Body {
    pub identity: ClosureId,
    pub source: NodeKey,
    /// Same-unit authenticated const wrapper; the ordinary child remains the body owner.
    pub constant: Option<crate::syntax::ast::NodeId>,
    pub body: NodeKey,
    pub parameters: List,
    pub signature: TypeId,
    /// Solved internal result before the lifetime-only formal signature fold.
    pub internal_result: Option<TypeId>,
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
    Reference,
    Index,
    Field,
    Pattern,
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
    /// Values formed from parent places before checked export to field aliases.
    pub formation_fields: Vec<TypeId>,
    pub accesses: HashMap<NodeKey, captures::PathId>,
    pub adjusted_accesses: HashMap<(NodeKey, AccessKind), AdjustedAccess>,
    pub nested_accesses: HashMap<(NodeKey, captures::PathId), captures::PathId>,
    /// Explicit non-reading uses still require initialized parent places.
    pub mentions: Vec<captures::PathId>,
    pub binding_mutability: HashMap<DefId, bool>,
}
#[derive(Default)]
struct ImportedClosures {
    entries: HashMap<ClosureId, ImportedClosure>,
}

struct ImportedClosure {
    body: Body,
    origin: crate::resolve::body_origin::BodyOrigin,
    public_binders: u32,
    constant: bool,
}

#[derive(Default)]
pub struct Bodies {
    pub(crate) inputs: HashMap<NodeKey, Vec<RegionId>>,
    pub(crate) values: Vec<Body>,
    pub(crate) sources: HashMap<NodeKey, usize>,
    identities: Vec<u32>,
    imports: Option<imports::Imports>,
    conditions: Option<execution_requirements::Catalog>,
    pub(crate) environments: HashMap<ClosureId, Environment>,
    imported: Option<Box<ImportedClosures>>,
}
impl Bodies {
    pub(crate) fn publication_growth(&self) -> crate::support::retained::Growth {
        // The retained pre-qualification library measured 72 bytes per row.
        // Charge the new witness slots for ordinary rows as well as const rows;
        // the pre-existing row/index allocation policy is otherwise unchanged.
        let mut growth = crate::support::retained::Growth::vector::<Body>(
            self.values.len(),
            self.values.capacity(),
            1,
        );
        growth.bytes =
            (growth.bytes / size_of::<Body>()).saturating_mul(size_of::<Body>().saturating_sub(72));
        growth
    }
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
        if let Some(index) = self
            .identities
            .get(identity.0 as usize)
            .and_then(|&slot| slot.checked_sub(1))
        {
            return self.values.get(index as usize);
        }
        self.imported
            .as_deref()?
            .entries
            .get(&identity)
            .map(|entry| &entry.body)
    }
    pub(crate) fn origin(
        &self,
        identity: ClosureId,
    ) -> Option<crate::resolve::body_origin::BodyOrigin> {
        if let Some(index) = self
            .identities
            .get(identity.0 as usize)
            .and_then(|&slot| slot.checked_sub(1))
        {
            return self
                .values
                .get(index as usize)
                .map(|body| body.semantic_source().into());
        }
        self.imported
            .as_deref()?
            .entries
            .get(&identity)
            .map(|entry| entry.origin)
    }
    pub(crate) fn is_const(&self, identity: ClosureId) -> bool {
        if let Some(index) = self
            .identities
            .get(identity.0 as usize)
            .and_then(|&slot| slot.checked_sub(1))
        {
            return self
                .values
                .get(index as usize)
                .is_some_and(|body| body.constant.is_some());
        }
        self.imported
            .as_deref()
            .and_then(|imported| imported.entries.get(&identity))
            .is_some_and(|entry| entry.constant)
    }
    pub(crate) fn check_const_source(
        &self,
        report: &crate::resolve::expansion::Report,
        identity: ClosureId,
    ) -> Result<bool, &'static str> {
        if let Some(index) = self
            .identities
            .get(identity.0 as usize)
            .and_then(|&slot| slot.checked_sub(1))
        {
            return self
                .values
                .get(index as usize)
                .ok_or("closure identity index is stale")?
                .check_const_source(report);
        }
        Ok(self.is_const(identity))
    }
    pub(crate) fn public_binders(&self, identity: ClosureId) -> Option<u32> {
        self.imported
            .as_deref()?
            .entries
            .get(&identity)
            .map(|entry| entry.public_binders)
    }
    pub(crate) fn insert_imported(
        &mut self,
        body: Body,
        origin: crate::resolve::body_origin::BodyOrigin,
        public_binders: u32,
        constant: bool,
    ) -> Result<(), &'static str> {
        if self.identity(body.identity).is_some() {
            return Err("imported closure body published twice");
        }
        let table = self.imported.get_or_insert_with(Default::default);
        table.entries.insert(
            body.identity,
            ImportedClosure {
                body,
                origin,
                public_binders,
                constant,
            },
        );
        Ok(())
    }
    pub(crate) fn insert(&mut self, body: Body) -> Result<(), &'static str> {
        let identity = body.identity.0 as usize;
        if self.sources.contains_key(&body.source)
            || self
                .identities
                .get(identity)
                .is_some_and(|&index| index != 0)
            || self
                .imported
                .as_deref()
                .is_some_and(|table| table.entries.contains_key(&body.identity))
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
