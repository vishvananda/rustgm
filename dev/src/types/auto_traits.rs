//! Auto-trait dependencies are keyed by both the type and the required marker.
mod structural;
use super::{ItemId, Store, TypeId, aggregate::Definitions};
use crate::support::hash::HashMap;

pub use crate::support::marker_traits::AutoTrait;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Obligation {
    pub ty: TypeId,
    pub marker: AutoTrait,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Ready(bool),
    Fields(ItemId),
    /// A projection or abstract subject requires caller-owned proof evidence.
    Proof(Obligation),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rule {
    Structural,
    Assumed,
    /// An explicit implementation must be selected before fields are inspected.
    External,
}
struct Node {
    obligation: Obligation,
    value: bool,
    parents: Option<usize>,
}
struct Edge {
    parent: usize,
    next: Option<usize>,
}

/// One proof scope owns this query and every external answer it supplies.
/// Positive cycle assumptions are never exported before all reachable fields
/// and external obligations have been checked. The query deliberately has no
/// cross-scope answer cache: projected proofs may carry lifetime witnesses.
pub struct Query {
    nodes: Vec<Node>,
    indices: HashMap<Obligation, usize>,
    edges: Vec<Edge>,
    cursor: usize,
    waiting: Option<Obligation>,
    result: Option<bool>,
    pub steps: usize,
}
impl Query {
    #[must_use]
    pub fn new(root: Obligation) -> Self {
        Self {
            nodes: vec![Node {
                obligation: root,
                value: true,
                parents: None,
            }],
            indices: HashMap::from_iter([(root, 0)]),
            edges: Vec::new(),
            cursor: 0,
            waiting: None,
            result: None,
            steps: 0,
        }
    }

    /// Traverse each immediate field dependency once, suspending at unfinished
    /// declarations or subjects requiring the caller's ordinary proof machinery.
    /// `rule` must use the same immutable environment throughout this query.
    /// # Errors
    /// Rejects malformed identities and exhausted graph/substitution work.
    pub fn advance(
        &mut self,
        types: &mut Store,
        definitions: &mut Definitions,
        mut rule: impl FnMut(&Store, Obligation) -> Result<Rule, &'static str>,
    ) -> Result<Step, &'static str> {
        if let Some(value) = self.result {
            return Ok(Step::Ready(value));
        }
        if let Some(obligation) = self.waiting {
            return Ok(Step::Proof(obligation));
        }
        while self.cursor < self.nodes.len() {
            self.charge(1)?;
            let obligation = self.nodes[self.cursor].obligation;
            match rule(types, obligation)? {
                Rule::Assumed => {}
                Rule::External => {
                    self.waiting = Some(obligation);
                    return Ok(Step::Proof(obligation));
                }
                Rule::Structural => {
                    if let Some(step) = self.expand(types, definitions, obligation)? {
                        return Ok(step);
                    }
                }
            }
            self.cursor += 1;
        }
        self.propagate()?;
        let result = self.nodes[0].value;
        self.result = Some(result);
        Ok(Step::Ready(result))
    }

    /// Retain the corresponding proof outside this structural query, including
    /// its lifetime constraints. An absent proof supplies `false`, never success.
    /// # Errors
    /// Rejects answers not matching the current suspended obligation.
    pub fn answer(&mut self, obligation: Obligation, value: bool) -> Result<(), &'static str> {
        if self.waiting != Some(obligation) {
            return Err("auto-trait answer does not match the pending obligation");
        }
        self.nodes[self.cursor].value = value;
        self.waiting = None;
        self.cursor += 1;
        Ok(())
    }

    /// A pending impl alias resolved to another nominal family. Reevaluate the
    /// rule without treating an unrelated header as positive or negative evidence.
    /// # Errors
    /// Rejects requests not matching the current suspended obligation.
    pub fn reconsider(&mut self, obligation: Obligation) -> Result<(), &'static str> {
        if self.waiting != Some(obligation) {
            return Err("auto-trait retry does not match the pending obligation");
        }
        self.waiting = None;
        Ok(())
    }

    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > 1_000_000 {
            return Err("auto-trait proof work budget exceeded");
        }
        Ok(())
    }

    fn dependency(&mut self, obligation: Obligation) -> Result<(), &'static str> {
        self.charge(1)?;
        let next = self.nodes.len();
        let child = *self.indices.entry(obligation).or_insert(next);
        if child == next {
            self.nodes.push(Node {
                obligation,
                value: true,
                parents: None,
            });
        }
        self.edges.push(Edge {
            parent: self.cursor,
            next: self.nodes[child].parents,
        });
        self.nodes[child].parents = Some(self.edges.len() - 1);
        Ok(())
    }

    fn propagate(&mut self) -> Result<(), &'static str> {
        let mut pending: Vec<_> = self
            .nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| (!node.value).then_some(index))
            .collect();
        while let Some(child) = pending.pop() {
            self.charge(1)?;
            let mut edge = self.nodes[child].parents;
            while let Some(index) = edge {
                self.charge(1)?;
                let entry = &self.edges[index];
                edge = entry.next;
                let parent = &mut self.nodes[entry.parent];
                if parent.value {
                    parent.value = false;
                    pending.push(entry.parent);
                }
            }
        }
        Ok(())
    }
}
