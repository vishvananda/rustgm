//! One paid canonical Field step owner preserves source identity and evaluation order.
use super::{CallContexts, Checker, Error, MAX_RETAINED, MAX_ROWS, MAX_TRANSIENT, SourceCallUse};
use crate::{
    sema::{
        bodies::{access_paths::OpenAccess, protocol_control::OpenPlan},
        model::Operation,
        protocol_plans::{
            self, Kind, Plan,
            access::{Range, Rows, Step},
        },
    },
    support::retained,
    syntax::ast::NodeId,
    types::infer::Solved,
};
use std::num::NonZeroU32;

impl CallContexts {
    fn access_growth(&self, count: usize) -> Result<retained::Growth, &'static str> {
        let (length, capacity) = self
            .access
            .as_deref()
            .map_or((0, 0), |rows| (rows.steps.len(), rows.steps.capacity()));
        if length.saturating_add(count) > MAX_ROWS {
            return Err("field checked-step inventory exhausted");
        }
        let mut growth = retained::Growth::vector::<Step>(length, capacity, count);
        if self.access.is_none() {
            growth.bytes = growth.bytes.saturating_add(size_of::<Rows>());
        }
        let current = self.bytes(self.rows.capacity())?;
        if current.saturating_add(growth.bytes) > MAX_TRANSIENT
            || current.saturating_add(growth.bytes) > MAX_RETAINED
        {
            return Err("field checked-step storage budget exceeded");
        }
        Ok(growth)
    }
}

impl Checker<'_> {
    pub(in crate::sema::bodies) fn publish_field_plan(
        &mut self,
        open: OpenPlan,
        field: OpenAccess,
        solved: &Solved,
        source_use: SourceCallUse,
    ) -> Result<Kind, Error> {
        let key = self.key(open.node);
        let count = field.steps.len().map_err(|e| self.error(open.node, e))?;
        let existing = match self.facts.node(key).and_then(|node| node.operation) {
            Some(
                Operation::Field { plan: Some(id), .. } | Operation::MethodReceiver { plan: id },
            ) => Some(
                id.current(self.facts, self.state.body, key)
                    .map_err(|e| self.error(open.node, e))?,
            ),
            _ => None,
        };
        let start = if let Some((_, _, _, steps)) = existing.and_then(|plan| plan.kind.access()) {
            steps.start
        } else {
            u32::try_from(
                self.facts
                    .call_contexts()
                    .and_then(CallContexts::access_steps)
                    .map_or(0, <[_]>::len),
            )
            .map_err(|_| self.error(open.node, "access checked-step index overflow"))?
        };
        let end = start
            .checked_add(
                u32::try_from(count)
                    .map_err(|_| self.error(open.node, "field step count overflow"))?,
            )
            .ok_or_else(|| self.error(open.node, "field checked-step range overflow"))?;
        let growth = self
            .facts
            .call_contexts()
            .ok_or_else(|| self.error(open.node, "field calls lost their context"))?
            .access_growth(if existing.is_some() { 0 } else { count })
            .map_err(|e| self.error(open.node, e))?;
        self.prepay_field_rows(open.node, count, growth)?;
        let (mut rows, mut order) = self.field_candidates(open, field, solved, source_use)?;
        order.sort_unstable_by_key(|&index| rows[index as usize].call.callable.0);
        let mut first = None;
        for (position, &ordinal) in order.iter().enumerate() {
            let id = start
                .checked_add(
                    u32::try_from(position)
                        .map_err(|_| self.error(open.node, "field row position overflow"))?,
                )
                .and_then(|index| index.checked_add(1))
                .and_then(NonZeroU32::new)
                .ok_or_else(|| self.error(open.node, "field row identity overflow"))?;
            if ordinal == 0 {
                first = Some(id);
            } else {
                rows[ordinal as usize - 1].next = Some(id);
            }
        }
        permute(&mut rows, &mut order).map_err(|e| self.error(open.node, e))?;
        let receiver = *solved
            .types
            .get(field.receiver.0 as usize)
            .ok_or_else(|| self.error(open.node, "access lost its solved receiver"))?;
        let steps = Range {
            start,
            end,
            first: first.ok_or_else(|| self.error(open.node, "access plan has no first step"))?,
        };
        let kind = match open.kind {
            super::super::protocol_control::OpenKind::Field(_) => Kind::Field {
                operand: field.operand,
                field: field.field.ok_or_else(|| {
                    self.error(open.node, "field publication lost its selected member")
                })?,
                receiver,
                dereferences: field.dereferences,
                steps,
            },
            super::super::protocol_control::OpenKind::Receiver { call, .. }
                if field.field.is_none() =>
            {
                Kind::Receiver {
                    operand: field.operand,
                    receiver,
                    dereferences: field.dereferences,
                    steps,
                    call: call.solved(solved).map_err(|e| self.error(open.node, e))?,
                }
            }
            _ => {
                return Err(self.error(open.node, "access publication lost its completed protocol"));
            }
        };
        let plan = Plan {
            node: key,
            source_use,
            kind,
        };
        protocol_plans::access::check_rows(self.report, self.facts, self.state.body, plan, &rows)?;
        self.commit_field_rows(open.node, existing, plan, rows)?;
        Ok(kind)
    }

    fn prepay_field_rows(
        &self,
        node: NodeId,
        count: usize,
        growth: retained::Growth,
    ) -> Result<(), Error> {
        let temporary = count.saturating_mul(size_of::<Step>() + size_of::<u32>());
        let current = self
            .facts
            .call_contexts()
            .ok_or_else(|| self.error(node, "field context disappeared"))?
            .bytes(
                self.facts
                    .call_contexts()
                    .map_or(0, |contexts| contexts.rows.capacity()),
            )
            .map_err(|e| self.error(node, e))?;
        if current
            .saturating_add(growth.bytes)
            .saturating_add(temporary)
            > MAX_TRANSIENT
        {
            return Err(self.error(
                node,
                "field canonicalization exceeds its paid overlap limit",
            ));
        }
        let sort_work =
            count.saturating_mul(count.checked_ilog2().map_or(1, |log| log as usize + 1));
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            count
                .saturating_mul(4)
                .saturating_add(sort_work)
                .saturating_add(growth.work),
            temporary.saturating_add(growth.bytes),
        )
    }

    fn field_candidates(
        &self,
        open: OpenPlan,
        field: OpenAccess,
        solved: &Solved,
        source_use: SourceCallUse,
    ) -> Result<(Vec<Step>, Vec<u32>), Error> {
        let count = field.steps.len().map_err(|e| self.error(open.node, e))?;
        let mut rows = Vec::new();
        let mut order = Vec::new();
        rows.try_reserve_exact(count)
            .map_err(|_| self.error(open.node, "field step allocation failed"))?;
        order
            .try_reserve_exact(count)
            .map_err(|_| self.error(open.node, "field order allocation failed"))?;
        if rows.capacity() > count || order.capacity() > count {
            return Err(self.error(
                open.node,
                "field canonicalization exceeded its paid capacity",
            ));
        }
        for index in 0..count {
            let step = self
                .state
                .call_types
                .access
                .as_deref()
                .and_then(|rows| rows.steps.get(field.steps.start as usize + index))
                .filter(|step| step.node == open.node)
                .ok_or_else(|| {
                    self.error(open.node, "field step belongs to another source node")
                })?;
            let methods = field
                .calls
                .ok_or_else(|| self.error(open.node, "field actual calls are unfinished"))?;
            let pending = self
                .state
                .call_types
                .methods
                .get(methods.method as usize + index)
                .filter(|(source, _)| *source == open.node)
                .map(|(_, pending)| pending)
                .ok_or_else(|| {
                    self.error(open.node, "field method belongs to another source node")
                })?;
            let (dereferences, borrow) = pending.capture_receiver();
            if dereferences != step.dereferences {
                return Err(self.error(open.node, "field changed its builtin reference prefix"));
            }
            let ty = |term: crate::types::infer::TermId| {
                solved
                    .types
                    .get(term.0 as usize)
                    .copied()
                    .ok_or_else(|| self.error(open.node, "field lost an actual solved type"))
            };
            rows.push(Step {
                source_use,
                node: open.node,
                call: self
                    .protocol_open_call(open, index)
                    .map_err(|e| self.error(open.node, e))?
                    .solved(solved)
                    .map_err(|e| self.error(open.node, e))?,
                subject: ty(step.subject)?,
                target: ty(step.target)?,
                output_view: ty(step
                    .output_view
                    .ok_or_else(|| self.error(open.node, "field output view is unfinished"))?)?,
                borrow: ty(borrow.ok_or_else(|| {
                    self.error(open.node, "field lost its actual operand borrow")
                })?)?,
                dereferences,
                next: None,
            });
            order.push(
                u32::try_from(index)
                    .map_err(|_| self.error(open.node, "field order index overflow"))?,
            );
        }
        Ok((rows, order))
    }

    fn commit_field_rows(
        &mut self,
        node: NodeId,
        existing: Option<Plan>,
        plan: Plan,
        rows: Vec<Step>,
    ) -> Result<(), Error> {
        let Some((_, _, _, steps)) = plan.kind.access() else {
            return Err(self.error(node, "field publication lost its kind"));
        };
        if let Some(existing) = existing {
            if existing != plan
                || protocol_plans::access::rows(self.facts, existing, self.state.body, steps)
                    .map_err(|e| self.error(node, e))?
                    != rows
            {
                return Err(self.error(
                    node,
                    "field replay changed its actual source, calls or checked steps",
                ));
            }
            return Ok(());
        }
        let report = self.report;
        let key = self.key(node);
        let owner = self.state.body;
        let contexts = CallContexts::slot(
            &mut self.facts.source_bindings,
            &mut self.facts.local_call_contexts,
        )
        .as_deref_mut()
        .ok_or_else(|| {
            crate::sema::model::Declarations::error(
                report,
                owner,
                "field publication lost its context",
            )
        })?;
        let steps_owner = contexts.access.get_or_insert_with(Default::default);
        if steps_owner.steps.len() != steps.start as usize {
            return Err(crate::sema::model::Declarations::error(
                report,
                key,
                "field step owner changed its append position",
            ));
        }
        steps_owner.steps.try_reserve(rows.len()).map_err(|_| {
            crate::sema::model::Declarations::error(
                report,
                key,
                "field checked-step allocation failed",
            )
        })?;
        steps_owner.steps.extend(rows);
        if contexts
            .bytes(contexts.rows.capacity())
            .map_err(|e| crate::sema::model::Declarations::error(report, key, e))?
            > MAX_RETAINED
        {
            return Err(crate::sema::model::Declarations::error(
                report,
                key,
                "field checked steps exceeded retained storage",
            ));
        }
        Ok(())
    }
}

fn permute(rows: &mut [Step], order: &mut [u32]) -> Result<(), &'static str> {
    if rows.len() != order.len() {
        return Err("field ordering changed its actual row count");
    }
    let mut moved = 0usize;
    for start in 0..rows.len() {
        if order[start] as usize == start {
            continue;
        }
        let mut cursor = start;
        let saved = rows[start];
        loop {
            moved = moved.saturating_add(1);
            if moved > rows.len() {
                return Err("field order is not a bounded permutation");
            }
            let next = order[cursor] as usize;
            if next >= rows.len() || next == cursor {
                return Err("field order leaves its actual permutation cycle");
            }
            order[cursor] = u32::try_from(cursor).map_err(|_| "field ordering index overflow")?;
            if next == start {
                rows[cursor] = saved;
                break;
            }
            rows[cursor] = rows[next];
            cursor = next;
        }
    }
    Ok(())
}
