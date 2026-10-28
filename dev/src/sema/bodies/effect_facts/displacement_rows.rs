//! Offset plans share the actual source-use owner and publish after all checks.
use super::{
    CallContexts, Checker, Error, MAX_RETAINED, MAX_ROWS, MAX_TRANSIENT, OffsetPlanId,
    SourceCallUse,
};
use crate::{
    resolve::unit::NodeKey,
    sema::{
        displacement_plans::{self, Plan},
        model::{Declarations, Operation},
    },
    support::{constness::Constness, retained},
    types::infer::Solved,
};
use std::num::NonZeroU32;

#[derive(Default)]
pub(super) struct Rows {
    pub rows: Vec<Plan>,
}
impl OffsetPlanId {
    pub(crate) fn current(
        self,
        facts: &Declarations,
        owner: NodeKey,
        node: NodeKey,
    ) -> Result<Plan, &'static str> {
        let plan = facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.call_contexts.as_deref())
            .and_then(|contexts| contexts.offsets.as_deref())
            .and_then(|rows| rows.rows.get(self.index() as usize))
            .copied()
            .ok_or("offset plan has no current source call context")?;
        if plan.node != node {
            return Err("offset plan belongs to another source node");
        }
        plan.source_use.effect_for(facts, owner)?;
        Ok(plan)
    }
}
impl CallContexts {
    fn offset_growth(&self, count: usize) -> Result<retained::Growth, &'static str> {
        let (length, capacity) = self
            .offsets
            .as_deref()
            .map_or((0, 0), |rows| (rows.rows.len(), rows.rows.capacity()));
        if length.saturating_add(count) > MAX_ROWS {
            return Err("offset plan inventory exhausted");
        }
        let mut growth = retained::Growth::vector::<Plan>(length, capacity, count);
        growth.bytes = growth
            .bytes
            .saturating_add(usize::from(self.offsets.is_none()) * size_of::<Rows>());
        let current = self.bytes(self.rows.capacity())?;
        if current.saturating_add(growth.bytes) > MAX_RETAINED.min(MAX_TRANSIENT) {
            return Err("offset plan storage budget exceeded");
        }
        Ok(growth)
    }
    fn reserve_offsets(&mut self, count: usize) -> Result<(), &'static str> {
        let growth = self.offset_growth(count)?;
        let old = self.bytes(self.rows.capacity())?;
        let rows = self.offsets.get_or_insert_with(Default::default);
        let length = rows.rows.len();
        let capacity = rows.rows.capacity();
        let additional = if length.saturating_add(count) > capacity {
            length
                .saturating_add(count)
                .max(capacity.saturating_mul(2))
                .max(4)
                - length
        } else {
            0
        };
        rows.rows
            .try_reserve_exact(additional)
            .map_err(|_| "offset plan allocation failed")?;
        if self.bytes(self.rows.capacity())? > old.saturating_add(growth.bytes) {
            return Err("offset plan allocation exceeds prepaid storage");
        }
        Ok(())
    }
}
impl Checker<'_> {
    pub(in crate::sema::bodies) fn replayed_offset_context(
        &self,
        key: NodeKey,
        operation: Option<Operation>,
        effect: Constness,
    ) -> Result<Option<SourceCallUse>, Error> {
        let Some(Operation::OffsetOf { plan }) = operation else {
            return Ok(None);
        };
        let plan =
            displacement_plans::checked(self.report, self.facts, self.state.body, key, plan)?;
        if plan
            .source_use
            .effect_for(self.facts, self.state.body)
            .map_err(|e| self.error(key.node, e))?
            != effect
        {
            return Err(self.error(
                key.node,
                "offset publication changed its current body effect",
            ));
        }
        Ok(Some(plan.source_use))
    }
    pub(in crate::sema::bodies) fn publish_displacement_plans(
        &mut self,
        solved: &Solved,
        source_use: Option<SourceCallUse>,
        allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let Some(pending) = self.state.call_types.offsets.as_deref() else {
            return Ok(());
        };
        let count = pending.rows.len();
        let source_use = source_use.ok_or_else(|| {
            self.error(
                self.state.body.node,
                "offset calls lost their source-use context",
            )
        })?;
        let contexts = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.call_contexts.as_deref())
            .ok_or_else(|| {
                self.error(
                    self.state.body.node,
                    "offset calls lost their current context",
                )
            })?;
        let growth = contexts
            .offset_growth(count)
            .map_err(|e| self.error(self.state.body.node, e))?;
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            growth.work.saturating_add(count),
            growth
                .bytes
                .saturating_add(count.saturating_mul(2 * size_of::<Plan>())),
        )?;
        let mut plans = Vec::new();
        plans.try_reserve_exact(count).map_err(|_| {
            self.error(
                self.state.body.node,
                "offset canonicalization allocation failed",
            )
        })?;
        for index in 0..count {
            let open = self
                .state
                .call_types
                .offsets
                .as_deref()
                .ok_or_else(|| self.error(self.state.body.node, "offset staging disappeared"))?
                .rows[index];
            let container = solved.types[open.container.0 as usize];
            let field_ty = solved.types[open.field_ty.0 as usize];
            let sized = self.body_is_sized(open.node, field_ty).map_err(|error| {
                self.error(
                    open.node,
                    format!("offset selected-field sizedness: {error}"),
                )
            })?;
            if !sized {
                return Err(self.error(open.node, "offset of unsized fields is not implemented"));
            }
            let call = open
                .call
                .solved(solved)
                .map_err(|e| self.error(open.node, e))?;
            let plan = Plan {
                node: self.key(open.node),
                source_use,
                container,
                field: open.field,
                call,
                module: self.state.module,
            };
            displacement_plans::validate(self.report, self.facts, plan)?;
            self.check_const_library_callable(open.node, call.callable, allowances)?;
            plans.push(plan);
        }
        self.commit_offsets(&plans)?;
        self.state.call_types.offsets = None;
        Ok(())
    }
    fn commit_offsets(&mut self, plans: &[Plan]) -> Result<(), Error> {
        let mut count = 0;
        for plan in plans {
            if let Some(Operation::OffsetOf { plan: id }) =
                self.facts.node(plan.node).and_then(|node| node.operation)
            {
                if id
                    .current(self.facts, self.state.body, plan.node)
                    .map_err(|e| self.error(plan.node.node, e))?
                    != *plan
                {
                    return Err(self.error(
                        plan.node.node,
                        "offset publication changed its existing plan",
                    ));
                }
            } else {
                count += 1;
            }
        }
        let report = self.report;
        let owner = self.state.body;
        let contexts = self
            .facts
            .source_bindings
            .as_deref_mut()
            .and_then(|catalog| catalog.call_contexts.as_deref_mut())
            .ok_or_else(|| {
                Declarations::error(report, owner, "offset publication lost its current context")
            })?;
        contexts
            .reserve_offsets(count)
            .map_err(|e| Declarations::error(report, owner, e))?;
        let rows = &mut contexts
            .offsets
            .as_deref_mut()
            .ok_or_else(|| Declarations::error(report, owner, "offset publication lost its rows"))?
            .rows;
        for plan in plans {
            let node =
                &mut self.facts.nodes[plan.node.file.0 as usize][plan.node.node.get() as usize];
            if matches!(node.operation, Some(Operation::OffsetOf { .. })) {
                continue;
            }
            let id = OffsetPlanId(
                NonZeroU32::new(u32::try_from(rows.len() + 1).map_err(|_| {
                    Declarations::error(report, owner, "offset plan identity exhausted")
                })?)
                .ok_or_else(|| {
                    Declarations::error(report, owner, "offset plan identity exhausted")
                })?,
            );
            rows.push(*plan);
            node.operation = Some(Operation::OffsetOf { plan: id });
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/displacement_members.rs"]
mod tests;
