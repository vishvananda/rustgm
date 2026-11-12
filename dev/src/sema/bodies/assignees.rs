//! A destructuring assignment is rustc's `{ let <pattern> = rhs; leaf = binding; ... }`.
//! Its tuple shells take the tuple of their elements' types, `_` takes a fresh
//! variable, and each place leaf is entered as the place an assignment writes.
//! The right operand then coerces to the assembled type.
use super::{Checker, Constructor, Domain, Error, Kind, NodeId, TermId, TypeId, Work};
use crate::syntax::assignee::{Assignee, Role};

impl Checker<'_> {
    /// Whether this assignment's left operand is a tuple or `_` assignee.
    pub(super) fn destructures(&self, left: NodeId) -> bool {
        self.report
            .package
            .unit(self.state.file)
            .configured
            .ast
            .destructures(left)
    }
    /// The assignee walk of `left`, in source order.
    pub(super) fn assignee_walk(&self, left: NodeId) -> Vec<Assignee> {
        let configured = &self.report.package.unit(self.state.file).configured;
        let mut walk = Vec::new();
        configured
            .ast
            .assignees(left, &configured.active, &mut walk);
        walk
    }
    /// Enter one assignee node. Shells finish after their elements.
    pub(super) fn enter_assignee(&mut self, node: NodeId) -> Result<(), Error> {
        match self.kind(node) {
            Kind::Group(inner) | Kind::Attributed { value: inner, .. } => {
                self.state.work.push(Work::AssigneeShell(node));
                self.state.work.push(Work::Assignee(inner));
            }
            Kind::Tuple(elements) => {
                self.state.work.push(Work::AssigneeShell(node));
                let unit = self.report.package.unit(self.state.file);
                for &element in unit.configured.ast.list(elements).iter().rev() {
                    if self.active(element) {
                        self.state.work.push(Work::Assignee(element));
                    }
                }
            }
            Kind::Wildcard => {
                let term = self.variable(node, Domain::Any)?;
                self.exit_expression(node, term)?;
            }
            Kind::Range {
                start: None,
                end: None,
                inclusive: false,
            } => {
                return Err(self.error(node, "rest assignees are not implemented"));
            }
            _ => {
                self.mark_union_write(node)?;
                self.mark_mutable_language_place(node)?;
                self.state.work.push(Work::Enter(node, None));
            }
        }
        Ok(())
    }
    /// A tuple shell's type is the tuple of its elements' types.
    pub(super) fn finish_assignee(&mut self, node: NodeId) -> Result<(), Error> {
        let term = match self.kind(node) {
            Kind::Group(inner) | Kind::Attributed { value: inner, .. } => self.term(inner)?,
            Kind::Tuple(elements) => {
                self.state.type_children.clear();
                let unit = self.report.package.unit(self.state.file);
                for &element in unit.configured.ast.list(elements) {
                    if self.active(element) {
                        let term = self.term(element)?;
                        self.state.type_children.push(term);
                    }
                }
                self.shape(node, Constructor::Tuple, None)?
            }
            _ => return Err(self.error(node, "invalid assignee shell")),
        };
        self.exit_expression(node, term)
    }
    /// Each place leaf must be assignable, as `leaf = value` is. The right
    /// operand coerces to the assembled assignee type.
    pub(super) fn destructuring_assignment(
        &mut self,
        node: NodeId,
        left: TermId,
        assignee: NodeId,
        right: NodeId,
    ) -> Result<TermId, Error> {
        for entry in self.assignee_walk(assignee) {
            if entry.role == Role::Place {
                self.assignment_target(node, entry.node)?;
            }
        }
        self.coerce(right, left)?;
        self.state.assignments.push((node, None, None));
        let right = self.term(right)?;
        let never = self.contextual_head(node, right)? == Some(Constructor::Never);
        self.known(node, if never { TypeId::NEVER } else { TypeId::UNIT })
    }
}
