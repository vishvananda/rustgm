//! Shared typed alternatives survive symbolic proof and runtime extent checking.
mod entailment;
mod probes;
use super::{Error, Graph, Region, Solution};
use crate::support::intern_slice::Slices;
pub(crate) use probes::Answers;
use std::collections::HashMap;

/// Identity in one graph's append-only verification arena.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct VerificationId(pub u32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Node {
    Outlives(Region, Region),
    All(u32),
    Any(u32),
}
#[derive(Default)]
pub(super) struct Verifications {
    nodes: Vec<Node>,
    index: HashMap<Node, VerificationId>,
    children: Slices<VerificationId>,
    requests: Vec<(VerificationId, u32)>,
    pub(super) verified: Vec<Option<bool>>,
    pub(super) cost: usize,
}
impl Graph {
    /// Form a test without adding an assumption or an inference constraint.
    /// LOCAL is a contract sentinel, not a source lifetime or runtime extent.
    /// # Errors
    /// Rejects foreign/sentinel identities and exhausted graph storage.
    pub fn verify_outlives(
        &mut self,
        longer: Region,
        shorter: Region,
    ) -> Result<VerificationId, Error> {
        self.kind(longer)?;
        self.kind(shorter)?;
        if longer == Region::LOCAL || shorter == Region::LOCAL {
            return Err(Error::Identity);
        }
        self.verification(Node::Outlives(longer, shorter))
    }
    /// Form a conjunction; an empty conjunction is true.
    /// # Errors
    /// Rejects foreign test identities and exhausted graph storage.
    pub fn verify_all(&mut self, children: &[VerificationId]) -> Result<VerificationId, Error> {
        let list = self.verification_children(children)?;
        self.verification(Node::All(list))
    }
    /// Form alternatives; an empty disjunction is false. No branch is assumed.
    /// # Errors
    /// Rejects foreign test identities and exhausted graph storage.
    pub fn verify_any(&mut self, children: &[VerificationId]) -> Result<VerificationId, Error> {
        let list = self.verification_children(children)?;
        self.verification(Node::Any(list))
    }
    fn verification_children(&mut self, children: &[VerificationId]) -> Result<u32, Error> {
        let count = self.verifications.as_ref().map_or(0, |v| v.nodes.len());
        if children.iter().any(|id| id.0 as usize >= count) {
            return Err(Error::Identity);
        }
        if children.is_empty() {
            return Ok(0);
        }
        if let Some(list) = self
            .verifications
            .as_ref()
            .and_then(|v| v.children.find(children))
        {
            return Ok(list);
        }
        // Reserve the list, its entries and the subsequent node before allocation.
        self.capacity(children.len().saturating_add(2))?;
        let storage = self.verifications.get_or_insert_with(Default::default);
        let list = storage
            .children
            .intern(children)
            .map_err(|_| Error::Budget)?;
        storage.cost += children.len().saturating_add(1);
        Ok(list)
    }
    fn verification(&mut self, node: Node) -> Result<VerificationId, Error> {
        if let Some(id) = self.verifications.as_ref().and_then(|v| v.index.get(&node)) {
            return Ok(*id);
        }
        self.capacity(1)?;
        let storage = self.verifications.get_or_insert_with(Default::default);
        let id = VerificationId(u32::try_from(storage.nodes.len()).map_err(|_| Error::Budget)?);
        storage.nodes.push(node);
        storage.index.insert(node, id);
        storage.cost += 1;
        Ok(id)
    }
    /// Require a typed verification after ordinary inference completes.
    /// Keep forced relationships as ordinary constraints when inference needs them.
    /// # Errors
    /// Rejects foreign tests and exhausted graph storage.
    pub fn require_verification(&mut self, id: VerificationId, source: u32) -> Result<(), Error> {
        self.capacity(1)?;
        let storage = self.verifications.as_mut().ok_or(Error::Identity)?;
        storage.nodes.get(id.0 as usize).ok_or(Error::Identity)?;
        storage.requests.push((id, source));
        storage.cost += 1;
        Ok(())
    }
}

trait Relations {
    fn charge(&mut self, count: usize) -> Result<(), Error>;
    fn outlives(&mut self, longer: Region, shorter: Region) -> Result<bool, Error>;
}
impl Verifications {
    fn children(&self, node: Node) -> Result<&[VerificationId], Error> {
        match node {
            Node::Outlives(..) => Ok(&[]),
            Node::All(id) | Node::Any(id) => self.children.get(id).ok_or(Error::Identity),
        }
    }
    fn check(
        &self,
        previous: Option<&[Option<bool>]>,
        context: &mut impl Relations,
    ) -> Result<Vec<Option<bool>>, Error> {
        if self.requests.is_empty() {
            return Ok(Vec::new());
        }
        context.charge(self.nodes.len())?;
        let mut values = vec![None; self.nodes.len()];
        let mut work = Vec::new();
        for &(root, source) in &self.requests {
            context.charge(1)?;
            self.evaluate_root(root, previous, &mut values, &mut work, context)?;
            if values[root.0 as usize] != Some(true) {
                return Err(Error::Verification { source });
            }
        }
        Ok(values)
    }
    fn evaluate_root(
        &self,
        root: VerificationId,
        previous: Option<&[Option<bool>]>,
        values: &mut [Option<bool>],
        work: &mut Vec<(VerificationId, bool)>,
        context: &mut impl Relations,
    ) -> Result<(), Error> {
        work.push((root, false));
        while let Some((id, finish)) = work.pop() {
            context.charge(1)?;
            let index = id.0 as usize;
            if values.get(index).ok_or(Error::Identity)?.is_some() {
                continue;
            }
            if let Some(previous) = previous {
                match previous.get(index).ok_or(Error::Identity)? {
                    Some(false) => {
                        values[index] = Some(false);
                        continue;
                    }
                    Some(true) => {}
                    None => return Err(Error::Identity),
                }
            }
            let node = self.nodes[index];
            let children = self.children(node)?;
            if !finish && !children.is_empty() {
                work.push((id, true));
                for &child in children.iter().rev() {
                    context.charge(1)?;
                    work.push((child, false));
                }
            } else {
                values[index] = Some(self.evaluate(node, values, context)?);
            }
        }
        Ok(())
    }
    fn evaluate(
        &self,
        node: Node,
        values: &[Option<bool>],
        context: &mut impl Relations,
    ) -> Result<bool, Error> {
        if let Node::Outlives(longer, shorter) = node {
            return context.outlives(longer, shorter);
        }
        let conjunction = matches!(node, Node::All(_));
        let mut result = conjunction;
        for &child in self.children(node)? {
            context.charge(1)?;
            let value = values
                .get(child.0 as usize)
                .copied()
                .flatten()
                .ok_or(Error::Identity)?;
            if conjunction {
                result &= value;
            } else {
                result |= value;
            }
        }
        Ok(result)
    }
}
struct Extents<F> {
    contains: F,
    steps: usize,
    limit: usize,
}
impl<F: FnMut(Region, Region) -> Result<bool, Error>> Relations for Extents<F> {
    fn charge(&mut self, count: usize) -> Result<(), Error> {
        self.steps = self.steps.checked_add(count).ok_or(Error::Budget)?;
        if self.steps > self.limit {
            return Err(Error::Budget);
        }
        Ok(())
    }
    fn outlives(&mut self, longer: Region, shorter: Region) -> Result<bool, Error> {
        (self.contains)(longer, shorter)
    }
}
impl Solution {
    /// Check additional region extents against the same symbolic alternatives.
    /// A branch must pass both phases; different successful branches cannot be
    /// combined into evidence that no individual branch supplies.
    /// Returns traversal work, excluding work performed by the caller's relation.
    /// # Errors
    /// Rejects failed conditions, malformed retained facts and exhausted work.
    pub fn verify_regions(
        &self,
        contains: impl FnMut(Region, Region) -> Result<bool, Error>,
        limit: usize,
    ) -> Result<usize, Error> {
        let Some(tests) = &self.graph.verifications else {
            return Ok(0);
        };
        let mut context = Extents {
            contains,
            steps: 0,
            limit,
        };
        tests.check(Some(&tests.verified), &mut context)?;
        Ok(context.steps)
    }
}
