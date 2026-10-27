//! Multi-call publication keeps one current body effect and exact protocol rows.
use super::{
    CallContexts, Checker, Error, MAX_RETAINED, MAX_ROWS, MAX_TRANSIENT, SourceCallUse, TryPlanId,
};
use crate::{
    resolve::unit::NodeKey,
    sema::{
        bodies::try_control::OpenKind,
        compiler_items::try_protocol,
        model::{Declarations, Operation},
        try_plans::{Kind, Plan},
    },
    types::{Kind as Type, infer::Solved},
};
use std::num::NonZeroU32;

impl TryPlanId {
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
            .and_then(|contexts| contexts.plans.get((self.0.get() - 1) as usize))
            .copied()
            .ok_or("Try plan has no current source call context")?;
        if plan.node != node {
            return Err("Try plan belongs to another source node");
        }
        plan.source_use.effect_for(facts, owner)?;
        Ok(plan)
    }
}
impl CallContexts {
    fn try_growth(&self, count: usize) -> Result<(usize, usize), &'static str> {
        let length = self
            .plans
            .len()
            .checked_add(count)
            .ok_or("Try plan inventory overflow")?;
        if length > MAX_ROWS {
            return Err("Try plan inventory exhausted");
        }
        if length <= self.plans.capacity() {
            return Ok((0, 0));
        }
        let bytes = length
            .checked_mul(2 * size_of::<Plan>())
            .ok_or("Try plan storage overflow")?;
        if self.bytes(self.rows.capacity())?.saturating_add(bytes) > MAX_TRANSIENT
            || self
                .rows
                .capacity()
                .saturating_mul(size_of::<super::Descriptor>())
                .saturating_add(bytes)
                .saturating_add(size_of::<Self>())
                > MAX_RETAINED
        {
            return Err("Try plan storage budget exceeded");
        }
        Ok((self.plans.len(), bytes))
    }
    fn reserve_try(&mut self, count: usize) -> Result<(), &'static str> {
        let (_, bytes) = self.try_growth(count)?;
        if bytes == 0 {
            return Ok(());
        }
        self.plans
            .try_reserve_exact(count)
            .map_err(|_| "Try plan allocation failed")?;
        if self.plans.capacity().saturating_mul(size_of::<Plan>()) > bytes
            || self.bytes(self.rows.capacity())? > MAX_RETAINED
        {
            return Err("Try plan allocation exceeds storage budget");
        }
        Ok(())
    }
}
impl Checker<'_> {
    pub(in crate::sema::bodies) fn is_try_call(&self, node: crate::syntax::ast::NodeId) -> bool {
        matches!(
            self.kind(node),
            crate::syntax::ast::Kind::TryBlock { .. }
                | crate::syntax::ast::Kind::Unary {
                    op: crate::syntax::ast::Unary::Try,
                    ..
                }
        )
    }
    pub(in crate::sema::bodies) fn replayed_try_context(
        &self,
        key: NodeKey,
        operation: Option<Operation>,
        effect: crate::support::constness::Constness,
    ) -> Result<Option<SourceCallUse>, Error> {
        let Some(Operation::Try { plan }) = operation else {
            return Ok(None);
        };
        let plan = plan
            .current(self.facts, self.state.body, key)
            .map_err(|e| Declarations::error(self.report, key, e))?;
        if plan
            .source_use
            .effect_for(self.facts, self.state.body)
            .map_err(|e| Declarations::error(self.report, key, e))?
            != effect
        {
            return Err(Declarations::error(
                self.report,
                key,
                "Try call publication changed its current body effect",
            ));
        }
        Ok(Some(plan.source_use))
    }
    pub(in crate::sema::bodies) fn publish_try_plans(
        &mut self,
        solved: &Solved,
        source_use: Option<SourceCallUse>,
        allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        if self.state.call_types.try_plans.is_empty() {
            return Ok(());
        }
        let source_use = source_use.ok_or_else(|| {
            self.error(
                self.state.body.node,
                "Try calls lost their actual source-use effect",
            )
        })?;
        let count = self.state.call_types.try_plans.len();
        crate::sema::header_attributes::charge_source_contract_query(self.facts, 1)?;
        let contexts = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| catalog.call_contexts.as_deref())
            .ok_or_else(|| {
                self.error(self.state.body.node, "Try calls lost their current context")
            })?;
        let (copy_work, growth) = contexts
            .try_growth(count)
            .map_err(|e| self.error(self.state.body.node, e))?;
        let bytes = count
            .checked_mul(2 * size_of::<Plan>())
            .and_then(|bytes| bytes.checked_add(growth))
            .ok_or_else(|| {
                self.error(
                    self.state.body.node,
                    "Try canonicalization storage overflow",
                )
            })?;
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            count.saturating_add(copy_work),
            bytes,
        )?;
        let mut plans = Vec::new();
        plans.try_reserve_exact(count).map_err(|_| {
            self.error(
                self.state.body.node,
                "Try canonicalization allocation failed",
            )
        })?;
        for index in 0..count {
            let open = self.state.call_types.try_plans[index];
            let kind = match open.kind {
                OpenKind::Block { body, call } => Kind::Block {
                    body,
                    from_output: call.solved(solved).map_err(|e| self.error(open.node, e))?,
                },
                OpenKind::Branch {
                    operand,
                    branch,
                    residual,
                    target,
                    control_flow,
                    continue_,
                    break_,
                } => Kind::Branch {
                    operand,
                    branch: branch
                        .solved(solved)
                        .map_err(|e| self.error(open.node, e))?,
                    residual: residual
                        .solved(solved)
                        .map_err(|e| self.error(open.node, e))?,
                    target,
                    control_flow: *solved.types.get(control_flow.0 as usize).ok_or_else(|| {
                        self.error(open.node, "Try branch lost its solved control-flow type")
                    })?,
                    continue_,
                    break_,
                },
            };
            let plan = Plan {
                node: self.key(open.node),
                source_use,
                kind,
            };
            self.check_try_plan(plan, allowances)?;
            plans.push(plan);
        }
        self.commit_try_plans(plans)?;
        self.state.call_types.try_plans = Vec::new();
        Ok(())
    }
    fn commit_try_plans(&mut self, plans: Vec<Plan>) -> Result<(), Error> {
        let mut new_count = 0;
        for plan in &plans {
            if let Some(Operation::Try { plan: id }) =
                self.facts.node(plan.node).and_then(|node| node.operation)
            {
                if id
                    .current(self.facts, self.state.body, plan.node)
                    .map_err(|e| self.error(plan.node.node, e))?
                    != *plan
                {
                    return Err(self.error(
                        plan.node.node,
                        "Try canonical publication changed its existing plan",
                    ));
                }
            } else {
                new_count += 1;
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
                Declarations::error(
                    report,
                    owner,
                    "Try publication lost its current call context",
                )
            })?;
        contexts
            .reserve_try(new_count)
            .map_err(|e| Declarations::error(report, owner, e))?;
        for plan in plans {
            if matches!(
                self.facts.nodes[plan.node.file.0 as usize][plan.node.node.get() as usize]
                    .operation,
                Some(Operation::Try { .. })
            ) {
                continue;
            }
            let id = TryPlanId(
                NonZeroU32::new(u32::try_from(contexts.plans.len() + 1).map_err(|_| {
                    Declarations::error(report, owner, "Try plan identity exhausted")
                })?)
                .ok_or_else(|| Declarations::error(report, owner, "Try plan identity exhausted"))?,
            );
            contexts.plans.push(plan);
            self.facts.nodes[plan.node.file.0 as usize][plan.node.node.get() as usize].operation =
                Some(Operation::Try { plan: id });
        }
        Ok(())
    }
    fn check_try_plan(
        &mut self,
        plan: Plan,
        allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        let protocol = try_protocol::source(self.report, self.facts, plan.node)?;
        let expected = match plan.kind {
            Kind::Block { .. } => [Some(protocol.from_output), None],
            Kind::Branch {
                target,
                control_flow,
                continue_,
                break_,
                ..
            } => {
                if !matches!(self.facts.types.kind(control_flow),Some(Type::Adt(owner,_)) if owner==protocol.control_flow)
                    || continue_ != protocol.continue_
                    || break_ != protocol.break_
                {
                    return Err(self.error(
                        plan.node.node,
                        "Try plan changed its actual control-flow enum or variants",
                    ));
                }
                self.prepare_storage(plan.node.node, control_flow)?;
                [
                    Some(protocol.branch),
                    Some(if target.is_some() {
                        protocol.into_try_type
                    } else {
                        protocol.from_residual
                    }),
                ]
            }
        };
        for (call, expected) in plan.calls().into_iter().zip(expected) {
            let Some(call) = call else {
                continue;
            };
            if !matches!(self.facts.types.kind(call.callable),Some(Type::FunctionItem {item,..}) if Some(crate::resolve::model::DefId(item.0))==expected)
            {
                return Err(self.error(
                    plan.node.node,
                    "Try plan changed its actual protocol callable",
                ));
            }
            self.check_const_library_callable(plan.node.node, call.callable, allowances)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/residual_protocol.rs"]
mod tests;
