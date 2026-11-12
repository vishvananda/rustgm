//! The left operand of a destructuring assignment is an assignee expression,
//! not a value (the Rust Reference's destructuring assignment). Tuples are
//! shells, `_` assigns nothing, and every other operand is a place leaf.
use super::ast::{Kind, NodeId, Syntax};

/// One node of an assignee walk, in source order: a shell precedes its
/// elements, and groups are transparent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Assignee {
    pub node: NodeId,
    /// The walk index of the enclosing tuple and this node's active field.
    pub parent: Option<(u32, u32)>,
    pub role: Role,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Tuple,
    Wildcard,
    /// A `..` rest marker; it assigns nothing, but it is not yet implemented.
    Rest,
    Place,
}

impl<L> Syntax<L> {
    /// Strip groups and attributes from an assignment's left operand.
    #[must_use]
    pub fn assignee_root(&self, mut left: NodeId) -> NodeId {
        while let Kind::Group(inner) | Kind::Attributed { value: inner, .. } = self.node(left).kind
        {
            left = inner;
        }
        left
    }
    /// Whether an assignment's left operand destructures: a tuple or `_`.
    #[must_use]
    pub fn destructures(&self, left: NodeId) -> bool {
        matches!(
            self.node(self.assignee_root(left)).kind,
            Kind::Tuple(_) | Kind::Wildcard
        )
    }
    /// Append `root`'s assignee walk to `output`, skipping inactive elements.
    pub fn assignees(&self, root: NodeId, active: &[bool], output: &mut Vec<Assignee>) {
        let mut pending = vec![(root, None)];
        while let Some((node, parent)) = pending.pop() {
            let node = self.assignee_root(node);
            let role = match self.node(node).kind {
                Kind::Tuple(_) => Role::Tuple,
                Kind::Wildcard => Role::Wildcard,
                Kind::Range {
                    start: None,
                    end: None,
                    inclusive: false,
                } => Role::Rest,
                _ => Role::Place,
            };
            let index = u32::try_from(output.len()).unwrap_or(u32::MAX);
            output.push(Assignee { node, parent, role });
            if let Kind::Tuple(elements) = self.node(node).kind {
                let start = pending.len();
                let mut field = 0;
                for &element in self.list(elements) {
                    if active.get(element.get() as usize).copied().unwrap_or(false) {
                        pending.push((element, Some((index, field))));
                        field += 1;
                    }
                }
                pending[start..].reverse();
            }
        }
    }
}
