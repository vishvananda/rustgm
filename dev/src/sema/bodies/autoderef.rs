//! Method discovery retains actual Deref steps before selecting their final calls.
use super::{Checker, Error, NodeId, TermId, Work};
use super::{
    access_paths::{AccessPlanId, OpenAccess, StepRange},
    protocol_control::{OpenCall, OpenKind, OpenPlan},
};
use crate::types::infer::Constructor;

impl Checker<'_> {
    pub(super) fn method_receiver_term(
        &mut self,
        node: NodeId,
        operand: NodeId,
    ) -> Result<TermId, Error> {
        let Some(id) = self.receiver_plan(node) else {
            return self.term(operand);
        };
        self.advance_projections(node)?;
        let plan = self
            .state
            .call_types
            .protocol_plans
            .get(id.index())
            .filter(|plan| plan.node == node)
            .ok_or_else(|| self.error(node, "method receiver belongs to another discovery"))?;
        let OpenKind::ReceiverDiscovery(access) = plan.kind else {
            return Err(self.error(node, "method receiver discovery already finished"));
        };
        if access.operand != operand {
            return Err(self.error(node, "method receiver changed its actual operand"));
        }
        Ok(access.receiver)
    }

    pub(super) fn advance_method_receiver(
        &mut self,
        node: NodeId,
        operand: NodeId,
        (term, (subject, dereferences)): (TermId, (TermId, u32)),
        expectation: super::Expectation,
    ) -> Result<bool, Error> {
        if !crate::sema::compiler_items::dereferencing::available(self.facts) {
            return Ok(false);
        }
        if self.head(node, subject)?.is_none() {
            return Ok(false);
        }
        let (owner, _) = crate::sema::compiler_items::dereferencing::target(
            self.report,
            self.facts,
            self.key(node),
        )?;
        let assumptions = self.trait_assumptions(node)?;
        if !crate::sema::traits::selection::may_have_subject(
            self.facts,
            &mut self.state.infer,
            crate::types::ItemId(owner.0),
            subject,
            assumptions.as_deref(),
        )
        .map_err(|e| self.error(node, e))?
        {
            return Ok(false);
        }
        self.builtin_method_dereferences(node, term, dereferences)?;
        let (index, target) = self.discover_field_target(node, subject, dereferences)?;
        let end = index
            .checked_add(1)
            .ok_or_else(|| self.error(node, "method receiver step range overflow"))?;
        if let Some(id) = self.receiver_plan(node) {
            let source = self.key(node);
            let report = self.report;
            let plan = self
                .state
                .call_types
                .protocol_plans
                .get_mut(id.index())
                .filter(|plan| plan.node == node)
                .ok_or_else(|| {
                    super::Declarations::error(
                        report,
                        source,
                        "method receiver lost its actual plan",
                    )
                })?;
            let OpenKind::ReceiverDiscovery(access) = &mut plan.kind else {
                return Err(self.error(node, "method receiver changed its discovery stage"));
            };
            if access.operand != operand || access.steps.end != index || access.calls.is_some() {
                return Err(self.error(node, "method receiver changed its actual contiguous steps"));
            }
            access.steps.end = end;
            access.receiver = target;
        } else {
            let slot = self.retain_try(OpenPlan {
                node,
                kind: OpenKind::ReceiverDiscovery(OpenAccess {
                    operand,
                    field: None,
                    receiver: target,
                    dereferences: 0,
                    steps: StepRange { start: index, end },
                    calls: None,
                }),
            })?;
            self.retain_receiver_plan(
                node,
                AccessPlanId::new(slot).map_err(|e| self.error(node, e))?,
            )?;
        }
        self.state
            .work
            .push(Work::MethodArguments(node, None, expectation));
        Ok(true)
    }

    pub(super) fn builtin_method_dereferences(
        &mut self,
        node: NodeId,
        mut term: TermId,
        count: u32,
    ) -> Result<TermId, Error> {
        let owned_box = self.facts.aggregates.owned_box();
        for _ in 0..count {
            if let Some(owner) = owned_box
                && matches!(
                    self.head(node, term)?,
                    Some(Constructor::Adt(item, _) | Constructor::AdtArgs(item, _)) if item == owner
                )
            {
                // Reading the box's leading pointer needs its exact storage layout.
                self.state.call_types.demand_fields(term);
            }
            term = self.child(node, term, 0)?;
        }
        Ok(term)
    }

    pub(super) fn complete_method_receiver(
        &mut self,
        node: NodeId,
        borrow: Option<TermId>,
        call: OpenCall,
    ) -> Result<(), Error> {
        let Some(id) = self.receiver_plan(node) else {
            return Ok(());
        };
        if let Some(borrow) = borrow
            && matches!(
                self.head(node, borrow)?,
                Some(Constructor::Reference { mutable: true, .. })
            )
        {
            self.mark_receiver_mutable(node)?;
        }
        let source = self.key(node);
        let report = self.report;
        let plan = self
            .state
            .call_types
            .protocol_plans
            .get_mut(id.index())
            .filter(|plan| plan.node == node)
            .ok_or_else(|| {
                super::Declarations::error(
                    report,
                    source,
                    "method receiver completion lost its actual plan",
                )
            })?;
        let OpenKind::ReceiverDiscovery(access) = plan.kind else {
            return Err(self.error(node, "method receiver completed twice"));
        };
        plan.kind = OpenKind::Receiver { access, call };
        Ok(())
    }

    pub(super) fn access_step_method(&self, node: NodeId, index: usize) -> bool {
        [
            self.access_plan(node),
            self.coercion_plan(node),
            self.index_plan(node),
        ]
        .into_iter()
        .flatten()
        .filter_map(|id| self.state.call_types.protocol_plans.get(id.index()))
        .filter_map(|plan| plan.kind.access())
        .any(|access| {
            access.calls.is_some_and(|range| {
                let start = range.method as usize;
                index >= start && access.steps.len().is_ok_and(|count| index - start < count)
            })
        })
    }
}
