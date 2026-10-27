//! Expression traversal preserves place-result scopes across transparent wrappers.
use super::{Context, Error, Kind, NodeId, Punctuation, Syntax, Walker, Work};
use crate::syntax::ast::BlockFlavor;

impl Walker<'_> {
    pub(super) fn expression(&mut self, node: NodeId, context: Context) -> Result<(), Error> {
        self.record(node, context)?;
        match self.kind(node) {
            Syntax::Block {
                flavor: BlockFlavor::Const,
                ..
            } if node != self.source.node => {}
            Syntax::Block {
                flavor: BlockFlavor::Plain | BlockFlavor::Unsafe | BlockFlavor::Const,
                statements,
                tail,
            } => {
                let mut inner = self.scope(node, Kind::Block, context)?;
                inner.block = inner.scope;
                if let Some(tail) = tail {
                    let mut tail_context = self.scope(tail, Kind::Tail, inner)?;
                    if !self.narrowed_temporary(node, true)? {
                        tail_context.temporary = context.temporary;
                    }
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
            } => self.if_scopes(node, condition, then, otherwise, context)?,
            Syntax::Loop { body } | Syntax::While { body, .. } => {
                let inner = self.scope(node, Kind::Iteration, context)?;
                self.push(Work::Expression(body, inner))?;
                if let Syntax::While { condition, .. } = self.kind(node) {
                    self.push(Work::Condition(condition, inner))?;
                }
            }
            Syntax::For { .. } => self.for_scopes(node, context)?,
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
    fn for_scopes(&mut self, node: NodeId, context: Context) -> Result<(), Error> {
        let Syntax::For {
            pattern,
            iterator,
            body,
        } = self.kind(node)
        else {
            return Err(self.error(node, "invalid for-loop scope source"));
        };
        let extent = self.scope(node, Kind::Block, context)?;
        let iteration = self.scope(node, Kind::Iteration, extent)?;
        self.push(Work::Expression(body, iteration))?;
        self.push(Work::Pattern(pattern, iteration.scope))?;
        self.push(Work::Expression(iterator, extent))?;
        Ok(())
    }
    fn assembly_scopes(&mut self, arguments: super::List, context: Context) -> Result<(), Error> {
        for index in (arguments.start..arguments.end).rev() {
            if let Syntax::AssemblyOperand { input, output, .. } = self.kind(self.child(index)) {
                if output != input
                    && let Some(output) = output
                {
                    self.push(Work::Expression(output, context))?;
                }
                if let Some(input) = input {
                    self.push(Work::Expression(input, context))?;
                }
            }
        }
        Ok(())
    }
    fn operands(&mut self, node: NodeId, kind: Syntax, context: Context) -> Result<(), Error> {
        match kind {
            Syntax::LibraryValue {
                kind: crate::support::library_types::LibraryValue::FormatSuperLet { .. },
                value: Some(value),
            } => {
                self.format_binding(node, value, context)?;
            }
            Syntax::TryBlock { body: value, .. }
            | Syntax::Group(value)
            | Syntax::Attributed { value, .. }
            | Syntax::Statement(value)
            | Syntax::Label { value, .. }
            | Syntax::Unary { operand: value, .. }
            | Syntax::Cast { value, .. }
            | Syntax::Field { value, .. }
            | Syntax::FieldInit { value, .. } => {
                self.push(Work::Expression(value, context))?;
            }
            Syntax::LibraryValue { value, .. }
            | Syntax::Return(value)
            | Syntax::Break { value, .. } => {
                if let Some(value) = value {
                    self.push(Work::Expression(value, context))?;
                }
            }
            Syntax::Repeat { value, .. } => self.push(Work::Expression(value, context))?,
            Syntax::Range { start, end, .. } => {
                if let Some(end) = end {
                    self.push(Work::Expression(end, context))?;
                }
                if let Some(start) = start {
                    self.push(Work::Expression(start, context))?;
                }
            }
            Syntax::InlineAssembly(arguments) => self.assembly_scopes(arguments, context)?,
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
            | Syntax::Name(..)
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
