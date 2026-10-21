//! Expression traversal preserves place-result scopes across transparent wrappers.
use super::{Context, Error, Kind, NodeId, Punctuation, Syntax, Walker, Work};
use crate::syntax::ast::BlockFlavor;

impl Walker<'_> {
    pub(super) fn expression(&mut self, node: NodeId, context: Context) -> Result<(), Error> {
        self.record(node, context)?;
        match self.kind(node) {
            Syntax::Block {
                flavor: BlockFlavor::Plain,
                statements,
                tail,
            } => {
                let mut inner = self.scope(node, Kind::Block, context)?;
                inner.block = inner.scope;
                if let Some(tail) = tail {
                    let tail_context = self.scope(tail, Kind::Tail, inner)?;
                    self.push(Work::Expression(tail, tail_context))?;
                }
                self.statements(statements, inner)?;
            }
            Syntax::Expansion { children, tail } => {
                if let Some(tail) = tail {
                    self.push(Work::Expression(tail, context))?;
                }
                self.statements(children, context)?;
            }
            Syntax::Let {
                pattern,
                value,
                otherwise,
                ..
            } => {
                self.push(Work::Pattern(pattern, context.block))?;
                if let Some(value) = value {
                    let lifetime = super::Lifetime::Scope(context.block);
                    self.push(Work::ExtendExpression(value, lifetime))?;
                    if self.extending_pattern(pattern)? {
                        self.push(Work::ExtendPlace(value, lifetime))?;
                    }
                    self.push(Work::Expression(value, context))?;
                }
                if let Some(otherwise) = otherwise {
                    // Failure does not enter the success bindings' extended lifetime.
                    self.push(Work::Expression(otherwise, context))?;
                }
            }
            Syntax::If {
                condition,
                then,
                otherwise,
            } => {
                let inner = self.scope(node, Kind::IfThen, context)?;
                if let Some(otherwise) = otherwise {
                    self.push(Work::Expression(otherwise, context))?;
                }
                self.push(Work::Expression(then, inner))?;
                self.push(Work::Condition(condition, inner))?;
            }
            Syntax::Loop { body } | Syntax::While { body, .. } => {
                let inner = self.scope(node, Kind::Iteration, context)?;
                self.push(Work::Expression(body, inner))?;
                if let Syntax::While { condition, .. } = self.kind(node) {
                    self.push(Work::Condition(condition, inner))?;
                }
            }
            Syntax::Match { value, arms } => {
                self.expressions(arms, context)?;
                // The scrutinee deliberately inherits its enclosing temporary scope.
                self.push(Work::Expression(value, context))?;
            }
            Syntax::Arm {
                pattern,
                guard,
                value,
            } => {
                let arm = self.scope(node, Kind::Arm, context)?;
                let inner = if let Some(guard) = guard {
                    let guard_context = self.scope(guard, Kind::Guard, arm)?;
                    self.push(Work::Condition(guard, guard_context))?;
                    guard_context
                } else {
                    arm
                };
                self.push(Work::Expression(value, inner))?;
                self.push(Work::Pattern(pattern, arm.scope))?;
            }
            Syntax::Binary {
                op: Punctuation::AndAnd | Punctuation::OrOr,
                left,
                right,
            } => {
                let right_context = self.scope(right, Kind::Condition, context)?;
                let left_context = self.scope(left, Kind::Condition, context)?;
                self.push(Work::Expression(right, right_context))?;
                self.push(Work::Expression(left, left_context))?;
            }
            other => self.operands(node, other, context)?,
        }
        Ok(())
    }
    fn operands(&mut self, node: NodeId, kind: Syntax, context: Context) -> Result<(), Error> {
        match kind {
            Syntax::Group(value)
            | Syntax::Attributed { value, .. }
            | Syntax::Statement(value)
            | Syntax::Label { value, .. }
            | Syntax::Unary { operand: value, .. }
            | Syntax::Cast { value, .. }
            | Syntax::Field { value, .. }
            | Syntax::FieldInit { value, .. } => {
                self.push(Work::Expression(value, context))?;
            }
            Syntax::Return(value) | Syntax::Break { value, .. } => {
                if let Some(value) = value {
                    self.push(Work::Expression(value, context))?;
                }
            }
            Syntax::Repeat { value, .. } => self.push(Work::Expression(value, context))?,
            Syntax::Tuple(elements) | Syntax::Array(elements) => {
                self.expressions(elements, context)?;
            }
            Syntax::Binary { left, right, .. }
            | Syntax::Index {
                value: left,
                index: right,
            } => {
                self.push(Work::Expression(right, context))?;
                self.push(Work::Expression(left, context))?;
            }
            Syntax::MethodCall {
                receiver: callee,
                arguments,
                ..
            }
            | Syntax::GenericMethodCall {
                receiver: callee,
                arguments,
                ..
            }
            | Syntax::Call { callee, arguments } => {
                self.expressions(arguments, context)?;
                self.push(Work::Expression(callee, context))?;
            }
            Syntax::Struct { fields, base, .. } => {
                if let Some(base) = base {
                    self.push(Work::Expression(base, context))?;
                }
                self.expressions(fields, context)?;
            }
            Syntax::Closure { .. }
            | Syntax::Literal(_)
            | Syntax::Name(_)
            | Syntax::Path { .. }
            | Syntax::Qualified { .. }
            | Syntax::Continue(_)
            | Syntax::EmptyStatement
            | Syntax::Item(_) => {}
            _ => return Err(self.error(node, "unsupported expression in temporary scopes")),
        }
        Ok(())
    }
}
