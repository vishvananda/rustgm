//! Definite control transfer is separate from an expression's contextual value type.
use super::{Checker, Constructor, Error, Kind, List, NodeId, Op};
impl Checker<'_> {
    pub(super) fn finish_divergence(
        &mut self,
        node: NodeId,
        local_exit: bool,
    ) -> Result<(), Error> {
        let never = self.head(node, self.term(node)?)? == Some(Constructor::Never);
        let transferred = if never || local_exit {
            never
        } else {
            match self.kind(node) {
                Kind::If {
                    condition,
                    then,
                    otherwise,
                } => {
                    self.diverges[condition.0 as usize]
                        || (self.diverges[then.0 as usize]
                            && otherwise.is_some_and(|n| self.diverges[n.0 as usize]))
                }
                Kind::LetCondition { value, .. }
                | Kind::Group(value)
                | Kind::Attributed { value, .. }
                | Kind::Label { value, .. }
                | Kind::Statement(value)
                | Kind::Cast { value, .. }
                | Kind::Unary { operand: value, .. }
                | Kind::Field { value, .. }
                | Kind::FieldInit { value, .. }
                | Kind::Repeat { value, .. } => self.diverges[value.0 as usize],
                Kind::Let { value, .. } => value.is_some_and(|n| self.diverges[n.0 as usize]),
                Kind::Binary {
                    op: Op::AndAnd | Op::OrOr,
                    left,
                    ..
                } => self.diverges[left.0 as usize],
                Kind::Binary { left, right, .. } => {
                    self.diverges[left.0 as usize] || self.diverges[right.0 as usize]
                }
                Kind::Index { value, index } => {
                    self.diverges[value.0 as usize] || self.diverges[index.0 as usize]
                }
                Kind::Tuple(elements) | Kind::Array(elements) => self.list_diverges(elements),
                Kind::MethodCall {
                    receiver: callee,
                    arguments,
                    ..
                }
                | Kind::GenericMethodCall {
                    receiver: callee,
                    arguments,
                    ..
                }
                | Kind::Call { callee, arguments } => {
                    self.diverges[callee.0 as usize] || self.list_diverges(arguments)
                }
                Kind::Struct { fields, base, .. } => {
                    self.list_diverges(fields) || base.is_some_and(|n| self.diverges[n.0 as usize])
                }
                Kind::Block {
                    statements, tail, ..
                }
                | Kind::Expansion {
                    children: statements,
                    tail,
                } => {
                    self.list_diverges(statements)
                        || tail.is_some_and(|n| self.diverges[n.0 as usize])
                }
                Kind::Match { value, arms } => {
                    self.diverges[value.0 as usize]
                        || self
                            .report
                            .package
                            .unit(self.file)
                            .configured
                            .ast
                            .list(arms)
                            .iter()
                            .filter(|n| self.active(**n))
                            .all(|n| self.diverges[n.0 as usize])
                }
                Kind::Arm { guard, value, .. } => {
                    guard.is_some_and(|n| self.diverges[n.0 as usize])
                        || self.diverges[value.0 as usize]
                }
                // Loop exits were checked against their own control frame, so an
                // internal break is not an escape from the enclosing expression.
                Kind::Loop { .. } | Kind::While { .. } => self.diverges[node.0 as usize],
                _ => false,
            }
        };
        self.diverges[node.0 as usize] = transferred;
        Ok(())
    }
    fn list_diverges(&self, nodes: List) -> bool {
        self.report
            .package
            .unit(self.file)
            .configured
            .ast
            .list(nodes)
            .iter()
            .any(|n| self.diverges[n.0 as usize])
    }
}
