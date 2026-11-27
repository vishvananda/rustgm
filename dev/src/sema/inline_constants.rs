//! Inline constants retain semantic body identity independently of runtime storage.
mod instances;
mod journals;
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::{
        body_origin::{BodyOrigin, ImportedBodyId},
        model::DefId,
        unit::{FileId, NodeKey},
    },
    sema::values::Constant,
    syntax::ast::NodeId,
    types::{TypeId, closure::ClosureId},
};
pub use instances::{Context, InlineInstance};
pub(crate) use journals::Lease as JournalLease;
use std::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(pub(super) u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evaluation {
    Required,
    SuppressedZeroRepeat,
    DeferredTemplate,
    DeferredUnreachable,
}

pub struct Body {
    pub source: NodeKey,
    pub origin: BodyOrigin,
    pub parent_body: NodeKey,
    pub parent_origin: BodyOrigin,
    /// Nearest enclosing non-inline body, retained once without ancestor rescans.
    context_owner: NodeKey,
    context_owner_origin: BodyOrigin,
    imported_closure_owner: Option<ClosureId>,
    pub declaration: Option<DefId>,
    pub result: Option<TypeId>,
    pub universals: u32,
    pub evaluation: Evaluation,
    pub value: Option<Constant>,
    pub root: Option<Box<RootFacts>>,
    parent: Option<Id>,
    first_child: Option<Id>,
    last_child: Option<Id>,
    next_sibling: Option<Id>,
}
impl Body {
    pub(crate) const fn context_owner(&self) -> NodeKey {
        self.context_owner
    }
    pub(crate) const fn context_owner_origin(&self) -> BodyOrigin {
        self.context_owner_origin
    }
    pub(crate) const fn imported_closure_owner(&self) -> Option<ClosureId> {
        self.imported_closure_owner
    }
}

pub struct RootFacts {
    pub node: super::model::NodeFacts,
    pub reference: Option<super::model::ReferenceAdjustment>,
    pub pointer: Option<TypeId>,
    pub unsizing: Option<crate::types::unsizing::Conversion<TypeId>>,
}

#[derive(Default)]
struct ImportedInline {
    by_body: HashMap<ImportedBodyId, Id>,
    owners: HashSet<ImportedBodyId>,
    children: HashMap<ImportedBodyId, Vec<ImportedBodyId>>,
}

/// This table exists only for crates containing inline bodies. Each source is
/// appended once; dependency and suppression lookups never scan prior rows.
#[derive(Default)]
pub struct Registry {
    bodies: Vec<Body>,
    sources: HashMap<NodeKey, Id>,
    owners: HashSet<NodeKey>,
    imported: Option<Box<ImportedInline>>,
    pub retained_bytes: usize,
    pub indexed: usize,
    pub lookups: Cell<usize>,
    pub preparation_visits: usize,
    pub publications: usize,
    pub value_publications: usize,
    pub instance_publications: usize,
    pub instance_lookups: Cell<usize>,
    pub suppression_visits: usize,
    instances: Option<Box<instances::Values>>,
}

impl Registry {
    fn lookup(&self, source: NodeKey) -> Option<Id> {
        self.lookups.set(self.lookups.get().saturating_add(1));
        self.sources.get(&source).copied()
    }

    pub(super) fn lookup_imported(&self, body: ImportedBodyId) -> Option<Id> {
        self.lookups.set(self.lookups.get().saturating_add(1));
        self.imported.as_deref()?.by_body.get(&body).copied()
    }

    fn lookup_origin(&self, origin: BodyOrigin) -> Option<Id> {
        match origin {
            BodyOrigin::Source(source) => self.lookup(source),
            BodyOrigin::Imported(body) => self.lookup_imported(body.id()),
        }
    }

    pub(crate) fn reserve(
        &mut self,
        source: NodeKey,
        parent_body: NodeKey,
        declaration: Option<DefId>,
        generic: bool,
    ) -> Result<Id, &'static str> {
        if self.lookup(source).is_some() {
            return Err("inline constant body was typed more than once");
        }
        if self.bodies.len() >= 1_000_000 {
            return Err("inline constant body limit exceeded");
        }
        let bytes = self
            .retained_bytes
            .saturating_add(2 * size_of::<Body>())
            .saturating_add(crate::support::retained::new_map::<NodeKey, Id>(1))
            .saturating_add(crate::support::retained::new_map::<NodeKey, ()>(1));
        if bytes > 64 * 1024 * 1024 {
            return Err("inline constant registry exceeds 64 MiB budget");
        }
        let id = Id(u32::try_from(self.bodies.len()).map_err(|_| "too many inline constants")?);
        let parent = self.lookup(parent_body);
        let evaluation = if parent.is_some_and(|parent| {
            self.bodies[parent.0 as usize].evaluation == Evaluation::SuppressedZeroRepeat
        }) {
            Evaluation::SuppressedZeroRepeat
        } else if generic {
            Evaluation::DeferredTemplate
        } else {
            Evaluation::DeferredUnreachable
        };
        let context_owner =
            parent.map_or(parent_body, |id| self.bodies[id.0 as usize].context_owner);
        self.bodies.push(Body {
            source,
            origin: BodyOrigin::Source(source),
            parent_body,
            parent_origin: BodyOrigin::Source(parent_body),
            context_owner,
            context_owner_origin: BodyOrigin::Source(context_owner),
            imported_closure_owner: None,
            declaration,
            result: None,
            universals: 0,
            evaluation,
            value: None,
            root: None,
            parent,
            first_child: None,
            last_child: None,
            next_sibling: None,
        });
        if let Some(parent) = parent {
            if let Some(last) = self.bodies[parent.0 as usize].last_child {
                self.bodies[last.0 as usize].next_sibling = Some(id);
            } else {
                self.bodies[parent.0 as usize].first_child = Some(id);
            }
            self.bodies[parent.0 as usize].last_child = Some(id);
        }
        self.sources.insert(source, id);
        self.indexed += 1;
        self.owners.insert(parent_body);
        self.retained_bytes = bytes;
        Ok(id)
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "imported anonymous header registration installs its complete checked provenance once"
    )]
    pub(crate) fn register_imported(
        &mut self,
        origin: BodyOrigin,
        parent_origin: BodyOrigin,
        imported_closure_owner: Option<ClosureId>,
        declaration: DefId,
        result: Option<TypeId>,
        universals: u32,
        deferred: bool,
    ) -> Result<Id, &'static str> {
        let BodyOrigin::Imported(body_origin) = origin else {
            return Err("imported inline constant requires an imported body origin");
        };
        let BodyOrigin::Imported(parent_id) = parent_origin else {
            return Err("imported inline constant requires an imported parent body");
        };
        if self.lookup_imported(body_origin.id()).is_some() {
            return Err("imported inline constant body was registered more than once");
        }
        if self.bodies.len() >= 1_000_000 {
            return Err("inline constant body limit exceeded");
        }
        let bytes = self
            .retained_bytes
            .saturating_add(2 * size_of::<Body>())
            .saturating_add(crate::support::retained::new_map::<ImportedBodyId, Id>(1))
            .saturating_add(crate::support::retained::new_map::<ImportedBodyId, ()>(1));
        if bytes > 64 * 1024 * 1024 {
            return Err("inline constant registry exceeds 64 MiB budget");
        }
        let id = Id(u32::try_from(self.bodies.len()).map_err(|_| "too many inline constants")?);
        let parent = self.lookup_imported(parent_id.id());
        let (context_owner_origin, closure_owner) = match parent {
            Some(parent_idx) => {
                let row = &self.bodies[parent_idx.0 as usize];
                (row.context_owner_origin, row.imported_closure_owner)
            }
            None => (parent_origin, imported_closure_owner),
        };
        let dummy = NodeKey {
            file: FileId(0),
            node: NodeId::new(0),
        };
        self.bodies.push(Body {
            source: dummy,
            origin,
            parent_body: dummy,
            parent_origin,
            context_owner: dummy,
            context_owner_origin,
            imported_closure_owner: closure_owner,
            declaration: Some(declaration),
            result,
            universals,
            evaluation: if deferred {
                Evaluation::DeferredTemplate
            } else {
                Evaluation::DeferredUnreachable
            },
            value: None,
            root: None,
            parent,
            first_child: None,
            last_child: None,
            next_sibling: None,
        });
        if let Some(parent) = parent {
            if let Some(last) = self.bodies[parent.0 as usize].last_child {
                self.bodies[last.0 as usize].next_sibling = Some(id);
            } else {
                self.bodies[parent.0 as usize].first_child = Some(id);
            }
            self.bodies[parent.0 as usize].last_child = Some(id);
        }
        let imported = self.imported.get_or_insert_with(Default::default);
        imported.by_body.insert(body_origin.id(), id);
        imported.owners.insert(parent_id.id());
        imported
            .children
            .entry(parent_id.id())
            .or_default()
            .push(body_origin.id());
        self.indexed += 1;
        if result.is_some() {
            self.publications += 1;
        }
        self.retained_bytes = bytes;
        Ok(id)
    }

    pub(crate) fn complete_imported(
        &mut self,
        origin: BodyOrigin,
        result: TypeId,
    ) -> Result<(), &'static str> {
        let BodyOrigin::Imported(body_origin) = origin else {
            return Err("imported inline completion requires an imported body origin");
        };
        let id = self
            .lookup_imported(body_origin.id())
            .ok_or("missing imported inline constant body")?;
        let body = self
            .bodies
            .get_mut(id.0 as usize)
            .ok_or("missing imported inline constant body")?;
        if let Some(existing) = body.result {
            if existing != result {
                return Err("imported inline constant result changed after publication");
            }
            return Ok(());
        }
        body.result = Some(result);
        self.publications += 1;
        Ok(())
    }

    pub(crate) fn complete(
        &mut self,
        id: Id,
        result: TypeId,
        universals: u32,
    ) -> Result<(), &'static str> {
        let body = self
            .bodies
            .get_mut(id.0 as usize)
            .ok_or("missing inline constant body")?;
        if body.result.is_some() {
            return Err("inline constant result was published more than once");
        }
        body.result = Some(result);
        body.universals = universals;
        self.publications += 1;
        Ok(())
    }
    pub(crate) fn retain_root(
        &mut self,
        source: NodeKey,
        root: RootFacts,
    ) -> Result<(), &'static str> {
        let id = self.lookup(source).ok_or("missing inline root owner")?;
        let bytes = self.retained_bytes.saturating_add(size_of::<RootFacts>());
        if bytes > 64 * 1024 * 1024 || self.bodies[id.0 as usize].root.is_some() {
            return Err("inline root is duplicate or exceeds storage budget");
        }
        self.bodies[id.0 as usize].root = Some(Box::new(root));
        self.retained_bytes = bytes;
        Ok(())
    }

    #[must_use]
    pub fn get(&self, source: NodeKey) -> Option<&Body> {
        self.lookup(source)
            .and_then(|id| self.bodies.get(id.0 as usize))
    }

    #[must_use]
    pub fn get_origin(&self, origin: BodyOrigin) -> Option<&Body> {
        self.lookup_origin(origin)
            .and_then(|id| self.bodies.get(id.0 as usize))
    }

    #[must_use]
    pub(crate) fn get_imported(&self, body: ImportedBodyId) -> Option<&Body> {
        self.lookup_imported(body)
            .and_then(|id| self.bodies.get(id.0 as usize))
    }

    #[must_use]
    pub fn has_children(&self, source: NodeKey) -> bool {
        self.lookups.set(self.lookups.get().saturating_add(1));
        self.owners.contains(&source)
    }

    #[must_use]
    pub fn has_children_origin(&self, origin: BodyOrigin) -> bool {
        match origin {
            BodyOrigin::Source(source) => self.has_children(source),
            BodyOrigin::Imported(body) => {
                self.lookups.set(self.lookups.get().saturating_add(1));
                self.imported
                    .as_deref()
                    .is_some_and(|table| table.owners.contains(&body.id()))
            }
        }
    }

    #[must_use]
    pub(crate) fn uncompleted_imported_children(&self, origin: BodyOrigin) -> Vec<ImportedBodyId> {
        let BodyOrigin::Imported(body) = origin else {
            return Vec::new();
        };
        let Some(imported) = self.imported.as_deref() else {
            return Vec::new();
        };
        let Some(children) = imported.children.get(&body.id()) else {
            return Vec::new();
        };
        children
            .iter()
            .copied()
            .filter(|child| {
                imported
                    .by_body
                    .get(child)
                    .and_then(|id| self.bodies.get(id.0 as usize))
                    .is_some_and(|entry| entry.result.is_none())
            })
            .collect()
    }
    #[must_use]
    pub(crate) fn at(&self, index: usize) -> Option<&Body> {
        self.bodies.get(index)
    }
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.bodies.len()
    }
    pub fn bodies(&self) -> impl Iterator<Item = &Body> {
        self.bodies.iter()
    }

    pub(crate) fn require_origin(
        &mut self,
        origin: BodyOrigin,
        result: TypeId,
    ) -> Result<(), &'static str> {
        let id = self
            .lookup_origin(origin)
            .ok_or("missing inline constant identity")?;
        let body = &mut self.bodies[id.0 as usize];
        if body.result != Some(result) {
            return Err("inline constant result type is not ready or changed");
        }
        match body.evaluation {
            Evaluation::DeferredTemplate => {
                return Err("generic inline constant execution requires anonymous specialization");
            }
            Evaluation::SuppressedZeroRepeat => {
                return Err("suppressed inline constant reached a value demand");
            }
            Evaluation::Required | Evaluation::DeferredUnreachable => {}
        }
        body.evaluation = Evaluation::Required;
        Ok(())
    }

    pub(crate) fn publish_origin(
        &mut self,
        origin: BodyOrigin,
        value: Constant,
    ) -> Result<(), &'static str> {
        let id = self
            .lookup_origin(origin)
            .ok_or("missing completed inline constant identity")?;
        let body = &mut self.bodies[id.0 as usize];
        if body.evaluation != Evaluation::Required {
            return Err("inline constant completed without a required value demand");
        }
        if body.value.is_some_and(|prior| prior != value) {
            return Err("inline constant value changed after publication");
        }
        if body.value.is_none() {
            self.value_publications += 1;
        }
        body.value = Some(value);
        Ok(())
    }

    /// Previously suppressed subtrees stop immediately, so nested zero repeats
    /// charge each child edge once rather than repeatedly walking descendants.
    pub(crate) fn suppress(&mut self, source: NodeKey) {
        let Some(id) = self.lookup(source) else {
            return;
        };
        let mut next = Some(id);
        while let Some(current) = next {
            self.suppression_visits += 1;
            let body = &mut self.bodies[current.0 as usize];
            let descend = body.evaluation != Evaluation::SuppressedZeroRepeat;
            body.evaluation = Evaluation::SuppressedZeroRepeat;
            if descend && body.first_child.is_some() {
                next = body.first_child;
                continue;
            }
            let mut cursor = current;
            loop {
                self.suppression_visits += 1;
                if cursor == id {
                    next = None;
                    break;
                }
                let body = &self.bodies[cursor.0 as usize];
                if let Some(sibling) = body.next_sibling {
                    next = Some(sibling);
                    break;
                }
                let Some(parent) = body.parent else {
                    next = None;
                    break;
                };
                cursor = parent;
            }
        }
    }
}
