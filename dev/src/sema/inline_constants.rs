//! Inline constants retain semantic body identity independently of runtime storage.
use crate::{
    resolve::{model::DefId, unit::NodeKey},
    sema::values::Constant,
    types::TypeId,
};
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Id(u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Evaluation {
    Required,
    SuppressedZeroRepeat,
    DeferredTemplate,
    DeferredUnreachable,
}

pub struct Body {
    pub source: NodeKey,
    pub parent_body: NodeKey,
    pub declaration: Option<DefId>,
    pub result: Option<TypeId>,
    pub evaluation: Evaluation,
    pub value: Option<Constant>,
    parent: Option<Id>,
    first_child: Option<Id>,
    last_child: Option<Id>,
    next_sibling: Option<Id>,
}

/// This table exists only for crates containing inline bodies. Each source is
/// appended once; dependency and suppression lookups never scan prior rows.
#[derive(Default)]
pub struct Registry {
    bodies: Vec<Body>,
    sources: HashMap<NodeKey, Id>,
    owners: HashSet<NodeKey>,
    pub retained_bytes: usize,
    pub indexed: usize,
    pub lookups: Cell<usize>,
    pub preparation_visits: usize,
    pub publications: usize,
    pub value_publications: usize,
    pub suppression_visits: usize,
}

impl Registry {
    fn lookup(&self, source: NodeKey) -> Option<Id> {
        self.lookups.set(self.lookups.get().saturating_add(1));
        self.sources.get(&source).copied()
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
        self.bodies.push(Body {
            source,
            parent_body,
            declaration,
            result: None,
            evaluation,
            value: None,
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

    pub(crate) fn complete(&mut self, id: Id, result: TypeId) -> Result<(), &'static str> {
        let body = self
            .bodies
            .get_mut(id.0 as usize)
            .ok_or("missing inline constant body")?;
        if body.result.is_some() {
            return Err("inline constant result was published more than once");
        }
        body.result = Some(result);
        self.publications += 1;
        Ok(())
    }

    #[must_use]
    pub fn get(&self, source: NodeKey) -> Option<&Body> {
        self.lookup(source)
            .and_then(|id| self.bodies.get(id.0 as usize))
    }

    #[must_use]
    pub fn has_children(&self, source: NodeKey) -> bool {
        self.lookups.set(self.lookups.get().saturating_add(1));
        self.owners.contains(&source)
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

    pub(crate) fn require(&mut self, source: NodeKey, result: TypeId) -> Result<(), &'static str> {
        let id = self
            .lookup(source)
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

    pub(crate) fn publish(&mut self, source: NodeKey, value: Constant) -> Result<(), &'static str> {
        let id = self
            .lookup(source)
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
