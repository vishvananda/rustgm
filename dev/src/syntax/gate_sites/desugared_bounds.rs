//! Real generated endpoint nodes share the existing cold source witness.
use super::{Declarations, NodeId, NodeKind, Syntax, error};
use crate::lex::token::LexError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Helper {
    Minimum,
    Maximum,
    Predecessor,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Endpoint {
    pub node: NodeId,
    pub expression: NodeId,
    pub root: NodeId,
    pub range: NodeId,
    pub base: NodeId,
    pub operand: Option<NodeId>,
    pub helper: Helper,
    pub invocation: Option<std::num::NonZeroU32>,
}
impl Endpoint {
    pub(crate) fn validate<L>(&self, ast: &Syntax<L>, ordinal: u32) -> Result<NodeId, LexError> {
        let invalid = || error(0, "pattern endpoint lost its actual owner or edge");
        if ast.nodes.get(self.node.get() as usize).map(|n| n.kind)
            != Some(NodeKind::PatternEndpoint { gate: ordinal })
        {
            return Err(invalid());
        }
        let Some(NodeKind::TypePattern {
            base,
            constraint: super::super::ast::TypeConstraint::Pattern(pattern),
        }) = ast.nodes.get(self.root.get() as usize).map(|n| n.kind)
        else {
            return Err(invalid());
        };
        if base != self.base {
            return Err(invalid());
        }
        let contains = if pattern == self.range {
            true
        } else if let Some(NodeKind::Pattern {
            flavor: super::super::ast::PatternFlavor::Or,
            path: None,
            elements: arms,
        }) = ast.nodes.get(pattern.get() as usize).map(|n| n.kind)
        {
            let arms = ast
                .edges
                .get(arms.start as usize..arms.end as usize)
                .ok_or_else(invalid)?;
            arms.len() == 2 && arms.contains(&self.range)
        } else {
            false
        };
        let Some(NodeKind::PatternRange {
            start,
            end,
            inclusive: true,
        }) = ast.nodes.get(self.range.get() as usize).map(|n| n.kind)
        else {
            return Err(invalid());
        };
        let selected = match self.helper {
            Helper::Minimum => start,
            Helper::Maximum | Helper::Predecessor => end,
        };
        if !contains || selected != Some(self.expression) {
            return Err(invalid());
        }
        if self.helper == Helper::Predecessor {
            let Some(NodeKind::Call { callee, arguments }) = ast
                .nodes
                .get(self.expression.get() as usize)
                .map(|n| n.kind)
            else {
                return Err(invalid());
            };
            if callee != self.node
                || ast
                    .edges
                    .get(arguments.start as usize..arguments.end as usize)
                    != Some(self.operand.as_slice())
            {
                return Err(invalid());
            }
        } else if self.expression != self.node || self.operand.is_some() {
            return Err(invalid());
        }
        Ok(base)
    }
}
impl Declarations {
    pub(crate) fn bounds(&self) -> &[Endpoint] {
        &self.bounds
    }
    pub(super) fn copy_bounds(&self, count: u32, to: &mut Self) -> Result<(), LexError> {
        let rows = self
            .bounds
            .get(..count as usize)
            .ok_or_else(|| error(0, "pattern endpoint prefix changed"))?;
        self.charge(rows.len(), std::mem::size_of_val(rows))
            .map_err(|e| error(0, e))?;
        to.bounds
            .try_reserve_exact(rows.len())
            .map_err(|_| error(0, "pattern endpoint prefix allocation failed"))?;
        to.bounds.extend_from_slice(rows);
        Ok(())
    }
}
impl<L> Syntax<L> {
    pub(crate) fn endpoint_growth(&self, incoming: usize) -> crate::support::retained::Growth {
        if incoming == 0 {
            return crate::support::retained::Growth::default();
        }
        let (length, capacity) = self
            .gate_sites
            .as_ref()
            .map_or((0, 0), |rows| (rows.bounds.len(), rows.bounds.capacity()));
        let mut growth =
            crate::support::retained::Growth::vector::<Endpoint>(length, capacity, incoming);
        if self.gate_sites.is_none() {
            growth.bytes = growth.bytes.saturating_add(size_of::<Declarations>());
        }
        growth
    }
    pub(crate) fn endpoint(&self, ordinal: u32) -> Option<&Endpoint> {
        self.gate_sites.as_ref()?.bounds.get(ordinal as usize)
    }
    pub(crate) fn next_endpoint(&self) -> Result<u32, LexError> {
        let count = self.gate_extent().bounds;
        if count >= 1_000_000 {
            return Err(error(0, "pattern endpoint budget exceeded"));
        }
        Ok(count)
    }
    pub(crate) fn record_endpoint(&mut self, row: Endpoint) -> Result<(), LexError> {
        let ordinal = self.next_endpoint()?;
        row.validate(self, ordinal)?;
        if self.gate_sites.is_none() {
            let rows = Declarations::default();
            rows.charge(1, size_of::<Declarations>())
                .map_err(|e| error(0, e))?;
            self.gate_sites = Some(Box::new(rows));
        }
        let rows = self
            .gate_sites
            .as_mut()
            .ok_or_else(|| error(0, "pattern endpoint owner missing"))?;
        let growth = crate::support::retained::Growth::vector::<Endpoint>(
            rows.bounds.len(),
            rows.bounds.capacity(),
            1,
        );
        rows.charge(growth.work.saturating_add(1), growth.bytes)
            .map_err(|e| error(0, e))?;
        rows.bounds
            .try_reserve(1)
            .map_err(|_| error(0, "pattern endpoint allocation failed"))?;
        rows.bounds.push(row);
        Ok(())
    }
    pub(crate) fn stamp_endpoints(&mut self, context: u32) -> Result<(), LexError> {
        let context = std::num::NonZeroU32::new(context)
            .ok_or_else(|| error(0, "pattern endpoint invocation missing"))?;
        if let Some(rows) = &mut self.gate_sites {
            rows.charge(rows.bounds.len(), 0).map_err(|e| error(0, e))?;
            for row in &mut rows.bounds {
                if row.root != self.root || row.invocation.is_some() {
                    return Err(error(0, "pattern endpoint belongs to a previous producer"));
                }
                row.invocation = Some(context);
            }
        }
        Ok(())
    }
    pub(super) fn move_bound_root(&mut self, from: NodeId, to: NodeId) {
        // A type pattern has at most two ranges and four endpoint edges.
        let mut gates = [None; 4];
        let mut used = 0;
        let Some(NodeKind::TypePattern {
            constraint: super::super::ast::TypeConstraint::Pattern(pattern),
            ..
        }) = self.nodes.get(from.get() as usize).map(|n| n.kind)
        else {
            return;
        };
        let mut ranges = [Some(pattern), None];
        if let Some(NodeKind::Pattern {
            flavor: super::super::ast::PatternFlavor::Or,
            path: None,
            elements: arms,
        }) = self.nodes.get(pattern.get() as usize).map(|n| n.kind)
            && let [a, b] = self.list(arms)
        {
            ranges = [Some(*a), Some(*b)];
        }
        for range in ranges.into_iter().flatten() {
            if let Some(NodeKind::PatternRange { start, end, .. }) =
                self.nodes.get(range.get() as usize).map(|n| n.kind)
            {
                for expression in [start, end].into_iter().flatten() {
                    let node = if let NodeKind::Call { callee, .. } =
                        self.nodes[expression.get() as usize].kind
                    {
                        callee
                    } else {
                        expression
                    };
                    if let NodeKind::PatternEndpoint { gate } = self.nodes[node.get() as usize].kind
                        && let Some(slot) = gates.get_mut(used)
                    {
                        *slot = Some(gate);
                        used += 1;
                    }
                }
            }
        }
        if let Some(rows) = &mut self.gate_sites {
            for gate in gates.into_iter().flatten() {
                if let Some(row) = rows
                    .bounds
                    .get_mut(gate as usize)
                    .filter(|r| r.root == from)
                {
                    row.root = to;
                }
            }
        }
    }
}
