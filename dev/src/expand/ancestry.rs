//! Actual invocation ancestry, independent of token marks and source provenance.
use super::marks::ExpansionId;

#[derive(Clone, Copy)]
struct Node {
    parent: ExpansionId,
    depth: u32,
    jump: ExpansionId,
}

/// An append-only tree with one skew-binary jump per invocation.
/// Physical source is the implicit root and requires no allocation.
#[derive(Default)]
pub struct Ancestry {
    nodes: Vec<Node>,
}
impl Ancestry {
    /// Record the actual invocation, after its parent has been recorded.
    ///
    /// Adjacent equal-length jumps combine into a jump of twice that length
    /// plus one. Otherwise this node starts a unit jump. Construction visits a
    /// constant number of records and never copies a parent's ancestor chain;
    /// Vec growth makes total append work amortized constant.
    ///
    /// # Errors
    /// Rejects nonconsecutive invocation identities, missing parents or overflow
    /// without changing the tree.
    pub fn append(&mut self, id: ExpansionId, parent: ExpansionId) -> Result<(), &'static str> {
        let expected = u32::try_from(self.nodes.len())
            .ok()
            .and_then(|len| len.checked_add(1))
            .ok_or("too many expansion ancestors")?;
        if id.0 != expected {
            return Err("expansion ancestry requires consecutive invocation identities");
        }
        let previous = self.node(parent).ok_or("unknown expansion parent")?;
        let middle = self.node(previous.jump).ok_or("unknown expansion jump")?;
        let end = self.node(middle.jump).ok_or("unknown expansion jump")?;
        let depth = previous
            .depth
            .checked_add(1)
            .ok_or("expansion depth overflow")?;
        let jump = if previous.depth - middle.depth == middle.depth - end.depth {
            middle.jump
        } else {
            parent
        };
        self.nodes.push(Node {
            parent,
            depth,
            jump,
        });
        Ok(())
    }

    /// Whether both identities exist and the first is an ancestor of the second.
    #[must_use]
    pub fn ancestor(&self, ancestor: ExpansionId, context: ExpansionId) -> bool {
        self.reaches(ancestor, context, || {})
    }

    // The jump lengths on a path form skew-binary blocks (2^k - 1).
    // Skip complete blocks; when a block straddles the requested depth, taking
    // its parent exposes its two smaller blocks. Only the block containing the
    // requested depth is split, so query work is logarithmic in path depth.
    // The observer counts this actual traversal in tests without retained state.
    fn reaches(
        &self,
        ancestor: ExpansionId,
        mut context: ExpansionId,
        mut visit: impl FnMut(),
    ) -> bool {
        let Some(target) = self.node(ancestor) else {
            return false;
        };
        let Some(mut current) = self.node(context) else {
            return false;
        };
        while current.depth > target.depth {
            visit();
            let Some(jump) = self.node(current.jump) else {
                return false;
            };
            context = if jump.depth >= target.depth {
                current.jump
            } else {
                current.parent
            };
            let Some(next) = self.node(context) else {
                return false;
            };
            current = next;
        }
        context == ancestor
    }

    fn node(&self, id: ExpansionId) -> Option<Node> {
        if id.0 == 0 {
            Some(Node {
                parent: id,
                depth: 0,
                jump: id,
            })
        } else {
            self.nodes.get(id.0 as usize - 1).copied()
        }
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/expansion_ancestry.rs"]
mod tests;
