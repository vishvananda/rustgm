//! One-parameter summaries share type, list and argument graphs across default bounds.
use super::dag_parts::{Node, Summary, parts};
use crate::support::hash::HashMap;
use crate::types::{
    Store,
    parameters::{
        ParameterId,
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
};
#[cfg(test)]
#[path = "../../../../tests/invariants/default_occurrences.rs"]
mod tests;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
enum Value {
    #[default]
    Empty,
    One(ParameterId),
    Multiple,
}
impl Value {
    fn merge(self, other: Self) -> Self {
        match (self, other) {
            (Self::Empty, value) | (value, Self::Empty) => value,
            (Self::One(left), Self::One(right)) if left == right => self,
            _ => Self::Multiple,
        }
    }
}

impl Summary for Value {
    fn empty() -> Self {
        Self::Empty
    }
    fn merge(self, other: Self) -> Self {
        Self::merge(self, other)
    }
}

#[derive(Default)]
pub(crate) struct Sole {
    completed: HashMap<Node, Value>,
    work: Vec<(Node, bool)>,
    pub steps: usize,
}

impl Sole {
    pub fn clause(
        &mut self,
        store: &Store,
        clause: Clause,
    ) -> Result<Option<ParameterId>, &'static str> {
        let predicate = match clause {
            Clause::UnstableFeature(id) => {
                self.charge(1)?;
                store
                    .feature_name(id)
                    .ok_or("invalid feature occurrence clause")?;
                return Ok(None);
            }
            Clause::Predicate(value) => value,
            Clause::ForAll(id) => {
                store
                    .quantified(id)
                    .ok_or("invalid default parameter clause")?
                    .predicate
            }
        };
        self.predicate(store, predicate)
    }

    /// Parameter identities are not bound by lifetime quantifiers. This is an
    /// occurrence summary, including non-injective projection inputs, never proof.
    pub fn predicate(
        &mut self,
        store: &Store,
        predicate: Predicate,
    ) -> Result<Option<ParameterId>, &'static str> {
        self.charge(1)?;
        let subject = self.collect(store, Node::Type(predicate.ty))?;
        let bound = match predicate.bound {
            Bound::Trait(id) | Bound::ConstTrait(id) | Bound::MaybeConstTrait(id) => {
                self.collect(store, Node::Trait(id))?
            }
            Bound::ProjectionEqual(result) => self.collect(store, Node::Type(result))?,
            Bound::Copy | Bound::Clone | Bound::Sized | Bound::Auto(_) | Bound::Outlives(_) => {
                Value::Empty
            }
        };
        Ok(match subject.merge(bound) {
            Value::One(id) => Some(id),
            Value::Empty | Value::Multiple => None,
        })
    }

    fn collect(&mut self, store: &Store, root: Node) -> Result<Value, &'static str> {
        self.work.push((root, false));
        while let Some((node, finish)) = self.work.pop() {
            self.charge(1)?;
            if self.completed.contains_key(&node) {
                continue;
            }
            if !finish {
                self.work.push((node, true));
            }
            let mut ready = true;
            let mut edges = 0;
            let value = parts::<true, _>(
                store,
                node,
                0,
                &mut edges,
                |id| Ok(Value::One(id)),
                |_, _| Ok(Value::Empty),
                |child, _| {
                    if let Some(value) = self.completed.get(&child) {
                        return Ok(*value);
                    }
                    if finish {
                        return Err("unfinished default parameter dependency");
                    }
                    if self.work.len() >= 1_000_000 {
                        return Err("default parameter occurrence work budget exceeded");
                    }
                    ready = false;
                    self.work.push((child, false));
                    Ok(Value::Empty)
                },
            )?;
            self.charge(edges)?;
            if ready {
                if !finish {
                    self.work.pop();
                }
                self.completed.insert(node, value);
            }
        }
        self.completed
            .get(&root)
            .copied()
            .ok_or("missing default parameter summary")
    }

    fn charge(&mut self, work: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(work);
        if self.steps.saturating_add(self.work.len()) > 1_000_000 {
            return Err("default parameter occurrence work budget exceeded");
        }
        Ok(())
    }
}
