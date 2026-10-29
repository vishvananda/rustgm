//! Desugared endpoints are actual expression nodes, never invented source text.
use super::{Kind, LexError, List, NodeId, Parser, error};
use crate::syntax::{
    ast::PatternFlavor,
    gate_sites::{Endpoint, Helper},
};

impl Parser<'_, '_> {
    pub(super) fn lower_pattern_endpoints(
        &mut self,
        root: NodeId,
        pattern: NodeId,
    ) -> Result<(), LexError> {
        if let Kind::Pattern {
            flavor: PatternFlavor::Or,
            path: None,
            elements,
        } = self.ast.node(pattern).kind
        {
            let arms = self.ast.list(elements);
            let [a, b] = arms else {
                return Err(error(
                    self.ast.node(pattern).span.start,
                    "only two scalar range alternatives have a complete consumer",
                ));
            };
            let (a, b) = (*a, *b);
            self.lower_range(root, a)?;
            self.lower_range(root, b)
        } else {
            self.lower_range(root, pattern)
        }
    }
    fn lower_range(&mut self, root: NodeId, range: NodeId) -> Result<(), LexError> {
        let Kind::PatternRange {
            mut start,
            mut end,
            inclusive,
        } = self.ast.node(range).kind
        else {
            return Err(error(
                self.ast.node(range).span.start,
                "pattern type requires real scalar ranges",
            ));
        };
        for bound in [start, end].into_iter().flatten() {
            if let Kind::Binding {
                name,
                symbol,
                mutable: false,
                by_ref: false,
                subpattern: None,
            } = self.ast.node(bound).kind
            {
                self.ast.nodes[bound.get() as usize].kind = Kind::Name(name, symbol);
            }
        }
        self.ast.nodes[range.get() as usize].kind = Kind::PatternRange {
            start,
            end,
            inclusive: true,
        };
        if start.is_none() {
            start = Some(self.generated_endpoint(root, range, Helper::Minimum, None)?);
        }
        if end.is_none() {
            end = Some(self.generated_endpoint(root, range, Helper::Maximum, None)?);
        } else if !inclusive {
            end = Some(self.generated_endpoint(root, range, Helper::Predecessor, end)?);
        }
        self.ast.nodes[range.get() as usize].kind = Kind::PatternRange {
            start,
            end,
            inclusive: true,
        };
        Ok(())
    }
    fn generated_endpoint(
        &mut self,
        root: NodeId,
        range: NodeId,
        helper: Helper,
        operand: Option<NodeId>,
    ) -> Result<NodeId, LexError> {
        let span = operand.map_or(self.ast.node(range).span, |n| self.ast.node(n).span);
        let gate = self.ast.next_endpoint()?;
        let node = self.node(span.start, Kind::PatternEndpoint { gate })?;
        self.ast.nodes[node.get() as usize].span = span;
        let expression = if let Some(operand) = operand {
            let start = u32::try_from(self.ast.edges.len())
                .map_err(|_| error(span.start, "endpoint argument identity overflow"))?;
            self.ast.edges.push(operand);
            let expression = self.node(
                span.start,
                Kind::Call {
                    callee: node,
                    arguments: List {
                        start,
                        end: start + 1,
                    },
                },
            )?;
            self.ast.nodes[expression.get() as usize].span = span;
            expression
        } else {
            node
        };
        let Kind::PatternRange {
            mut start, mut end, ..
        } = self.ast.node(range).kind
        else {
            return Err(error(span.start, "endpoint range changed"));
        };
        match helper {
            Helper::Minimum => start = Some(expression),
            Helper::Maximum | Helper::Predecessor => end = Some(expression),
        }
        self.ast.nodes[range.get() as usize].kind = Kind::PatternRange {
            start,
            end,
            inclusive: true,
        };
        let Kind::TypePattern { base, .. } = self.ast.node(root).kind else {
            return Err(error(span.start, "pattern endpoint base owner changed"));
        };
        self.ast.record_endpoint(Endpoint {
            node,
            expression,
            root,
            range,
            base,
            operand,
            helper,
            invocation: None,
        })?;
        Ok(expression)
    }
}
