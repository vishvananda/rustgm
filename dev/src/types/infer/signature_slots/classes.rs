//! Assigned structural equations retain one monotone history even without type links.
use super::{Class, Error, TermId};
use std::collections::HashMap;

struct Node {
    parent: TermId,
    rank: u8,
    class: Class,
}
#[derive(Default)]
pub(super) struct Classes {
    nodes: HashMap<TermId, Node>,
}
impl Classes {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.nodes);
        total
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn contains_key(&self, id: TermId) -> bool {
        self.nodes.contains_key(&id)
    }
    pub fn fresh(&mut self, id: TermId, class: Class) -> Result<(), Error> {
        if self.nodes.contains_key(&id) {
            return Err(Error::InvalidIdentity);
        }
        self.nodes.insert(
            id,
            Node {
                parent: id,
                rank: 0,
                class,
            },
        );
        Ok(())
    }
    fn root(&mut self, id: TermId) -> Result<(TermId, usize), Error> {
        let mut root = id;
        let mut steps = 0;
        loop {
            steps += 1;
            let Some(node) = self.nodes.get(&root) else {
                break;
            };
            if node.parent == root {
                break;
            }
            root = node.parent;
            if steps > self.nodes.len() {
                return Err(Error::InvalidIdentity);
            }
        }
        let mut cursor = id;
        while cursor != root {
            steps += 1;
            let node = self.nodes.get_mut(&cursor).ok_or(Error::InvalidIdentity)?;
            let next = node.parent;
            node.parent = root;
            cursor = next;
        }
        Ok((root, steps))
    }
    pub fn class(&mut self, id: TermId, fallback: Class) -> Result<(Class, usize), Error> {
        let (root, steps) = self.root(id)?;
        Ok((
            self.nodes.get(&root).map_or(fallback, |node| node.class),
            steps,
        ))
    }
    pub fn merge_value(&mut self, id: TermId, class: Class) -> Result<usize, Error> {
        let (root, steps) = self.root(id)?;
        if let Some(node) = self.nodes.get_mut(&root) {
            node.class = node.class.merge(class);
        } else {
            self.fresh(id, class)?;
        }
        Ok(steps + 1)
    }
    pub fn unite(
        &mut self,
        a: TermId,
        b: TermId,
        defaults: (Class, Class),
    ) -> Result<usize, Error> {
        if !self.nodes.contains_key(&a) {
            self.fresh(a, defaults.0)?;
        }
        if !self.nodes.contains_key(&b) {
            self.fresh(b, defaults.1)?;
        }
        let (a, first) = self.root(a)?;
        let (b, second) = self.root(b)?;
        if a == b {
            return Ok(first + second);
        }
        let left = self.nodes.get(&a).ok_or(Error::InvalidIdentity)?;
        let right = self.nodes.get(&b).ok_or(Error::InvalidIdentity)?;
        let class = left.class.merge(right.class);
        let (parent, child) = if left.rank >= right.rank {
            (a, b)
        } else {
            (b, a)
        };
        let increment = left.rank == right.rank;
        self.nodes
            .get_mut(&child)
            .ok_or(Error::InvalidIdentity)?
            .parent = parent;
        let node = self.nodes.get_mut(&parent).ok_or(Error::InvalidIdentity)?;
        node.class = class;
        node.rank = node
            .rank
            .checked_add(u8::from(increment))
            .ok_or(Error::Overflow)?;
        Ok(first + second + 2)
    }
}
