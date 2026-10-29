//! Expression gates share the existing source witness and cumulative budget.
use super::{Declarations, NodeId, NodeKind, Span, Syntax, error};
use crate::lex::token::LexError;

/// Both writers cap sparse columns and check every append's compact identity.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct Prefix {
    pub(crate) items: u32,
    pub(crate) expressions: u32,
    pub(crate) bounds: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExpressionKind {
    TryBlock,
    ConstClosure,
    OffsetOf,
}
impl ExpressionKind {
    pub(crate) fn keyword(self) -> &'static str {
        match self {
            Self::TryBlock => "try",
            Self::ConstClosure => "const",
            Self::OffsetOf => "offset_of",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Expression {
    pub(crate) node: NodeId,
    pub(crate) body: NodeId,
    pub(crate) keyword: Span,
    pub(crate) construct: Span,
    pub(super) context: Option<(std::num::NonZeroU32, std::num::NonZeroU32)>,
}
impl Expression {
    pub(crate) fn kind<L>(self, ast: &Syntax<L>) -> Result<ExpressionKind, LexError> {
        match ast
            .nodes
            .get(self.node.get() as usize)
            .map(|node| node.kind)
        {
            Some(NodeKind::TryBlock { .. }) => Ok(ExpressionKind::TryBlock),
            Some(NodeKind::ConstClosure { .. }) => Ok(ExpressionKind::ConstClosure),
            Some(NodeKind::OffsetOf { .. }) => Ok(ExpressionKind::OffsetOf),
            _ => Err(error(
                self.keyword.start,
                "expression-gate node kind changed",
            )),
        }
    }
    pub(crate) fn context_key(self) -> Option<(std::num::NonZeroU32, std::num::NonZeroU32)> {
        self.context
    }
    pub(crate) fn validate<L>(self, ast: &Syntax<L>, ordinal: u32) -> Result<NodeId, LexError> {
        let invalid = || error(self.keyword.start, "expression-gate source site changed");
        let node = ast
            .nodes
            .get(self.node.get() as usize)
            .ok_or_else(invalid)?;
        if let NodeKind::OffsetOf {
            container,
            fields,
            expression,
            gate,
        } = node.kind
        {
            let ty = ast
                .nodes
                .get(container.get() as usize)
                .ok_or_else(invalid)?;
            let path = ast
                .nodes
                .get(expression.get() as usize)
                .ok_or_else(invalid)?;
            if gate != ordinal
                || expression != self.body
                || path.span != self.construct
                || node.span.start >= self.keyword.start
                || self.keyword.end.checked_sub(self.keyword.start) != Some(9)
                || self.keyword.end > ty.span.start
                || ty.span.end > path.span.start
                || path.span.end >= node.span.end
            {
                return Err(invalid());
            }
            super::super::offset_fields::visit(ast, fields, |_, _| Ok(()))?;
            return Ok(expression);
        }
        let (NodeKind::TryBlock { body, gate }
        | NodeKind::ConstClosure {
            closure: body,
            gate,
        }) = node.kind
        else {
            return Err(invalid());
        };
        let block = ast.nodes.get(body.get() as usize).ok_or_else(invalid)?;
        let plain = if let NodeKind::Attributed { value, .. } = block.kind {
            ast.nodes.get(value.get() as usize).ok_or_else(invalid)?
        } else {
            block
        };
        let kind = self.kind(ast)?;
        let valid_construct = match kind {
            ExpressionKind::TryBlock => matches!(
                plain.kind,
                NodeKind::Block {
                    flavor: crate::syntax::ast::BlockFlavor::Plain,
                    ..
                }
            ),
            ExpressionKind::ConstClosure => matches!(
                block.kind,
                NodeKind::Closure {
                    asynchronous: false,
                    ..
                }
            ),
            ExpressionKind::OffsetOf => false,
        };
        if gate != ordinal
            || body != self.body
            || block.span != self.construct
            || !valid_construct
            || node.span.start != self.keyword.start
            || self.keyword.end.checked_sub(self.keyword.start)
                != Some(u32::try_from(kind.keyword().len()).map_err(|_| invalid())?)
            || self.keyword.end > block.span.start
            || node.span.end != block.span.end
        {
            return Err(invalid());
        }
        Ok(body)
    }
}
impl Declarations {
    #[cfg(test)]
    pub(crate) fn expression_storage(&self) -> (usize, usize, usize, usize) {
        let (steps, bytes) = self.witness.usage();
        (
            self.expressions.len(),
            self.expressions.capacity(),
            steps,
            bytes,
        )
    }
    pub(crate) fn extent(&self) -> Prefix {
        // Every writer checks <= 1_000_000 before publication. An invalid private
        // state saturates to a prefix which later checked replay must reject.
        Prefix {
            items: u32::try_from(self.entries.len()).unwrap_or(u32::MAX),
            expressions: u32::try_from(self.expressions.len()).unwrap_or(u32::MAX),
            bounds: u32::try_from(self.bounds.len()).unwrap_or(u32::MAX),
        }
    }
    pub(crate) fn expressions(&self) -> &[Expression] {
        &self.expressions
    }
    pub(super) fn copy_expressions(&self, count: u32, to: &mut Self) -> Result<(), LexError> {
        let entries = self
            .expressions
            .get(..count as usize)
            .ok_or_else(|| error(0, "invalid original expression-gate prefix"))?;
        self.charge(count as usize, std::mem::size_of_val(entries))
            .map_err(|message| error(0, message))?;
        to.expressions
            .try_reserve_exact(entries.len())
            .map_err(|_| error(0, "original expression-gate allocation failed"))?;
        to.expressions.extend_from_slice(entries);
        to.expressions_completed = self.expressions_completed.min(entries.len());
        Ok(())
    }
    pub(crate) fn pending_expression(&self) -> Option<(u32, Expression)> {
        self.expressions
            .get(self.expressions_completed)
            .map(|entry| {
                (
                    u32::try_from(self.expressions_completed).unwrap_or(u32::MAX),
                    *entry,
                )
            })
    }
    pub(crate) fn seal_expression(
        &mut self,
        ordinal: u32,
        expected: Expression,
        context: (std::num::NonZeroU32, std::num::NonZeroU32),
    ) -> Result<(), &'static str> {
        self.charge(1, 0)?;
        if ordinal as usize != self.expressions_completed {
            return Err("expression-gate pending ordinal changed");
        }
        let entry = self
            .expressions
            .get_mut(ordinal as usize)
            .ok_or("expression-gate row disappeared")?;
        if *entry != expected || entry.context.is_some() {
            return Err("expression-gate pending source changed");
        }
        entry.context = Some(context);
        self.expressions_completed += 1;
        Ok(())
    }
}
impl<L> Syntax<L> {
    pub(crate) fn gate_extent(&self) -> Prefix {
        self.gate_sites
            .as_ref()
            .map_or_else(Prefix::default, |entries| entries.extent())
    }
    pub(crate) fn expression_gate(&self, ordinal: u32) -> Option<&Expression> {
        self.gate_sites.as_ref()?.expressions.get(ordinal as usize)
    }
    pub(crate) fn next_expression_gate(&self) -> Result<u32, LexError> {
        let count = self
            .gate_sites
            .as_ref()
            .map_or(0, |entries| entries.expressions.len());
        if count >= 1_000_000 {
            return Err(error(0, "expression-gate budget exceeded"));
        }
        u32::try_from(count).map_err(|_| error(0, "expression-gate identity overflow"))
    }
    pub(crate) fn record_expression_gate(
        &mut self,
        node: NodeId,
        keyword: Span,
    ) -> Result<(), LexError> {
        let ordinal = self.next_expression_gate()?;
        let Some(
            NodeKind::TryBlock { body, .. }
            | NodeKind::ConstClosure { closure: body, .. }
            | NodeKind::OffsetOf {
                expression: body, ..
            },
        ) = self.nodes.get(node.get() as usize).map(|node| node.kind)
        else {
            return Err(error(
                keyword.start,
                "expression-gate requires actual gated node",
            ));
        };
        let block = self
            .nodes
            .get(body.get() as usize)
            .ok_or_else(|| error(keyword.start, "expression-gate block disappeared"))?
            .span;
        let entry = Expression {
            node,
            body,
            keyword,
            construct: block,
            context: None,
        };
        entry.validate(self, ordinal)?;
        if self.gate_sites.is_none() {
            let entries = Declarations::default();
            entries
                .charge(1, size_of::<Declarations>())
                .map_err(|message| error(keyword.start, message))?;
            self.gate_sites = Some(Box::new(entries));
        }
        let entries = self
            .gate_sites
            .as_mut()
            .ok_or_else(|| error(keyword.start, "expression-gate owner missing"))?;
        let growth = crate::support::retained::Growth::vector::<Expression>(
            entries.expressions.len(),
            entries.expressions.capacity(),
            1,
        );
        entries
            .charge(1_usize.saturating_add(growth.work), growth.bytes)
            .map_err(|message| error(keyword.start, message))?;
        entries
            .expressions
            .try_reserve(1)
            .map_err(|_| error(keyword.start, "expression-gate allocation failed"))?;
        entries.expressions.push(entry);
        Ok(())
    }
}
