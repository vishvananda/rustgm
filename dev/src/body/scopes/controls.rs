//! Statement boundaries, conditional bindings, and explicit pattern ownership.
use super::{Context, Error, Kind, List, NodeId, Punctuation, Syntax, Walker, Work};
impl Walker<'_> {
    pub(super) fn bind_scope(
        &mut self,
        node: NodeId,
        definition: crate::resolve::model::DefId,
        scope: super::Id,
    ) -> Result<(), Error> {
        if self
            .facts
            .scratch
            .bindings
            .insert(definition, scope)
            .is_some_and(|old| old != scope)
        {
            return Err(self.error(node, "binding has inconsistent storage scope"));
        }
        Ok(())
    }
    pub(super) fn statements(&mut self, list: List, context: Context) -> Result<(), Error> {
        for index in (list.start..list.end).rev() {
            self.push(Work::Statement(self.child(index), context))?;
        }
        Ok(())
    }
    pub(super) fn statement(&mut self, node: NodeId, context: Context) -> Result<(), Error> {
        if let Syntax::Expansion { children, tail } = self.kind(node) {
            self.record(node, context)?;
            if let Some(tail) = tail {
                self.push(Work::Statement(tail, context))?;
            }
            self.statements(children, context)
        } else {
            let inner = self.scope(node, Kind::Statement, context)?;
            self.push(Work::Expression(node, inner))
        }
    }
    pub(super) fn condition(&mut self, node: NodeId, context: Context) -> Result<(), Error> {
        match self.kind(node) {
            Syntax::LetCondition { pattern, value } => {
                self.record(node, context)?;
                self.push(Work::Pattern(pattern, context.scope))?;
                self.push(Work::Expression(value, context))
            }
            Syntax::Binary {
                op: Punctuation::AndAnd,
                left,
                right,
            } => {
                self.record(node, context)?;
                self.push(Work::Condition(right, context))?;
                self.push(Work::Condition(left, context))
            }
            Syntax::Group(value) | Syntax::Attributed { value, .. } => {
                self.record(node, context)?;
                self.push(Work::Condition(value, context))
            }
            _ => {
                let inner = self.scope(node, Kind::Condition, context)?;
                self.push(Work::Expression(node, inner))
            }
        }
    }
    pub(super) fn pattern(&mut self, node: NodeId, scope: super::Id) -> Result<(), Error> {
        match self.kind(node) {
            Syntax::Binding { subpattern, .. } => {
                let key = super::NodeKey {
                    node,
                    ..self.source
                };
                // A bare identifier pattern can resolve to a constant or unit constructor.
                if self
                    .types
                    .node(key)
                    .is_some_and(|n| n.binding.is_none() && n.reference.is_some())
                {
                    return Ok(());
                }
                let definition = self
                    .types
                    .node(key)
                    .and_then(|n| n.binding)
                    .ok_or_else(|| self.error(node, "pattern lacks selected binding"))?;
                self.bind_scope(node, definition, scope)?;
                if let Some(pattern) = subpattern {
                    self.push(Work::Pattern(pattern, scope))?;
                }
            }
            Syntax::Pattern { elements, .. }
            | Syntax::StructPattern {
                fields: elements, ..
            } => {
                for index in (elements.start..elements.end).rev() {
                    self.push(Work::Pattern(self.child(index), scope))?;
                }
            }
            Syntax::PatternField { pattern, .. }
            | Syntax::ReferencePattern { pattern, .. }
            | Syntax::Group(pattern)
            | Syntax::Attributed { value: pattern, .. } => {
                self.push(Work::Pattern(pattern, scope))?;
            }
            _ => {}
        }
        Ok(())
    }
}
