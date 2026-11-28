//! Call syntax selects a receiver permission before argument checking and retains its proof.
use super::{
    Checker, Constructor, Dependency, Error, Expectation, List, NodeId, TermId, Work,
    access_paths::{AccessPlanId, OpenAccess, StepRange},
    deref_coercions::SidePlan,
    protocol_control::{OpenKind, OpenPlan},
};
use crate::{
    sema::{
        invocations::Invocation,
        model::BodyRegionOrigin,
        traits::selection::{self, Caller, Selection},
    },
    support::call_traits::CallTrait,
    types::{
        ItemId,
        infer::{Context, Domain, Solved, arguments::Argument, predicates::Predicate},
    },
};

pub(super) enum Progress {
    Ready { signature: TermId, callee: TermId },
    Continue,
    Wait(Dependency),
}

pub(super) struct Pending {
    kind: Option<CallTrait>,
    dereferences: u32,
    borrow: Option<TermId>,
    pub receiver: TermId,
}
impl Pending {
    pub(super) const fn capture_receiver(&self) -> (Option<CallTrait>, u32) {
        (self.kind, self.dereferences)
    }
}
impl Checker<'_> {
    pub(super) fn invocation_plan(&self, node: NodeId) -> Option<AccessPlanId> {
        self.state
            .call_types
            .access
            .as_deref()?
            .invocations
            .get(&node)
            .copied()
    }

    pub(super) fn invocation_signature(
        &mut self,
        node: NodeId,
        callee: NodeId,
        arguments: List,
        expectation: Expectation,
    ) -> Result<Progress, Error> {
        let (mut subject, stepped) = if let Some(id) = self.invocation_plan(node) {
            self.advance_projections(node)?;
            let subject = self.invocation_plan_receiver(node, id)?;
            if let Some(dependency) = self.receiver_readiness(node, subject)? {
                self.state.work.push(Work::CallArguments(node, expectation));
                return Ok(Progress::Wait(dependency));
            }
            (subject, true)
        } else {
            (self.term(callee)?, false)
        };
        let mut dereferences = 0_u32;
        loop {
            self.state
                .infer
                .account_work(1)
                .map_err(|e| self.error(node, e))?;
            let head = self.head(node, subject)?;
            if matches!(head, Some(Constructor::Reference { .. })) {
                // Call syntax follows the referent; a bound on the reference
                // alone is not a callable permission on that referent.
                subject = self.child(node, subject, 0)?;
                dereferences = dereferences
                    .checked_add(1)
                    .ok_or_else(|| self.error(node, "call receiver depth exhausted"))?;
                continue;
            }

            if let Some(Constructor::Closure(identity, _)) = head {
                let kind = match self.facts.types.closure_kind(identity) {
                    Some(crate::types::closure::CallKind::Shared) => CallTrait::Fn,
                    Some(crate::types::closure::CallKind::Mutable) => CallTrait::FnMut,
                    Some(crate::types::closure::CallKind::Once) => CallTrait::FnOnce,
                    None => return Err(self.error(node, "closure call permission is unresolved")),
                };
                let signature = self.child(node, subject, 0)?;
                self.finish_invocation_target(node, callee, subject, dereferences, Some(kind))?;
                return Ok(Progress::Ready {
                    signature: self.call_signature(node, signature)?,
                    callee: subject,
                });
            }

            let signature = match head {
                Some(Constructor::Function { .. }) => Some(subject),
                Some(Constructor::FunctionItem(..) | Constructor::FunctionItemArgs(..)) => {
                    Some(self.child(node, subject, 0)?)
                }
                _ => None,
            };
            if let Some(signature) = signature {
                self.finish_invocation_target(node, callee, subject, dereferences, None)?;
                return Ok(Progress::Ready {
                    signature: self.call_signature(node, signature)?,
                    callee: subject,
                });
            }
            if head.is_none() {
                return Err(self.error(node, "call requires a known function type"));
            }
            if let Some((kind, tuple)) = self.select_invocation(node, subject, head)? {
                let signature = self.invocation_inputs(node, subject, tuple, arguments)?;
                self.finish_invocation_target(node, callee, subject, dereferences, Some(kind))?;
                return Ok(Progress::Ready {
                    signature,
                    callee: subject,
                });
            }
            if crate::sema::compiler_items::dereferencing::available(self.facts)
                && self.may_dereference_reference(node, subject)?
            {
                if dereferences == 0 && !stepped {
                    self.check_union_receiver(node, callee)?;
                }
                let (step, target) = self.discover_field_target(node, subject, dereferences)?;
                self.extend_invocation_discovery(node, callee, step, target)?;
                self.state.work.push(Work::CallArguments(node, expectation));
                return Ok(Progress::Continue);
            }
            return Err(self.error(node, "call requires a known function type"));
        }
    }

    fn finish_invocation_target(
        &mut self,
        node: NodeId,
        callee: NodeId,
        subject: TermId,
        dereferences: u32,
        kind: Option<CallTrait>,
    ) -> Result<(), Error> {
        let stepped = self.invocation_plan(node);
        if kind == Some(CallTrait::FnMut) {
            self.mark_mutable_language_place(callee)?;
            if stepped.is_some() {
                self.mark_receiver_mutable(node)?;
            }
        }
        let retain_derefs = if let Some(id) = stepped {
            self.complete_invocation_discovery(node, id, subject, dereferences)?;
            0
        } else {
            dereferences
        };
        self.retain_invocation(node, callee, subject, retain_derefs, kind)
    }

    fn invocation_plan_receiver(&self, node: NodeId, id: AccessPlanId) -> Result<TermId, Error> {
        match self
            .state
            .call_types
            .protocol_plans
            .get(id.index())
            .filter(|plan| plan.node == node)
            .map(|plan| plan.kind)
        {
            Some(OpenKind::Invocation {
                access,
                complete: false,
            }) => Ok(access.receiver),
            _ => Err(self.error(node, "call callee discovery lost its actual open plan")),
        }
    }

    fn extend_invocation_discovery(
        &mut self,
        node: NodeId,
        operand: NodeId,
        index: u32,
        target: TermId,
    ) -> Result<(), Error> {
        let end = index
            .checked_add(1)
            .ok_or_else(|| self.error(node, "call callee step range overflow"))?;
        let Some(id) = self.invocation_plan(node) else {
            let slot = self.retain_try(OpenPlan {
                node,
                kind: OpenKind::Invocation {
                    access: OpenAccess {
                        operand,
                        field: None,
                        receiver: target,
                        dereferences: 0,
                        steps: StepRange { start: index, end },
                        calls: None,
                    },
                    complete: false,
                },
            })?;
            let id = AccessPlanId::new(slot).map_err(|e| self.error(node, e))?;
            return self.retain_side_plan(node, id, SidePlan::Invocation);
        };
        match self
            .state
            .call_types
            .protocol_plans
            .get_mut(id.index())
            .filter(|plan| plan.node == node)
            .map(|plan| &mut plan.kind)
        {
            Some(OpenKind::Invocation {
                access,
                complete: false,
            }) if access.operand == operand && access.steps.end == index => {
                access.steps.end = end;
                access.receiver = target;
                Ok(())
            }
            _ => Err(self.error(
                node,
                "call callee discovery changed its source or contiguous steps",
            )),
        }
    }

    fn complete_invocation_discovery(
        &mut self,
        node: NodeId,
        id: AccessPlanId,
        receiver: TermId,
        dereferences: u32,
    ) -> Result<(), Error> {
        match self
            .state
            .call_types
            .protocol_plans
            .get_mut(id.index())
            .filter(|plan| plan.node == node)
            .map(|plan| &mut plan.kind)
        {
            Some(OpenKind::Invocation { access, complete })
                if !*complete && access.calls.is_none() =>
            {
                access.receiver = receiver;
                access.dereferences = dereferences;
                *complete = true;
                Ok(())
            }
            _ => Err(self.error(node, "call callee discovery completed twice")),
        }
    }

    fn invocation_effect(&self) -> Result<crate::support::constness::Constness, Error> {
        self.callee_use_effect()
    }

    fn select_invocation(
        &mut self,
        node: NodeId,
        subject: TermId,
        head: Option<Constructor>,
    ) -> Result<Option<(CallTrait, TermId)>, Error> {
        let assumptions = self.trait_assumptions(node)?.unwrap_or_default();
        let environment = self.trait_query_environment(node)?;
        for kind in CallTrait::ALL {
            let Some(item) = self.facts.associated.callable_definition(kind) else {
                continue;
            };
            if let Some(Constructor::Parameter(id)) = head
                && !assumptions.parameter_traits(id).contains(&ItemId(item.0))
            {
                continue;
            }
            if !self.possible_call_kind(node, subject, ItemId(item.0), &assumptions)? {
                continue;
            }
            // Only the chosen kind creates caller variables. Rejected alternatives
            // must not leave unconstrained tuple holes in the body's inference graph.
            let tuple = self.variable(node, Domain::Any)?;
            let goal = Predicate::trait_application_with_effect(
                subject,
                ItemId(item.0),
                vec![Argument::Type(tuple)],
                self.invocation_effect()?,
            );
            let first_region = self.trait_region_start(node)?;
            let selected = selection::select_available(
                self.facts,
                &mut self.state.infer,
                &goal,
                Caller {
                    assumptions: &assumptions,
                    environment: environment.as_ref(),
                    first_region,
                    retain_sized: false,
                    nesting: crate::sema::traits::selection::Nesting::ROOT,
                    placeholders: None,
                },
                self.availability,
            )
            .map_err(|e| self.error(node, e))?;
            match selected {
                Selection::Proven(evidence) => {
                    self.retain_trait_evidence(node, evidence)?;
                }
                Selection::Refuted(refutation) => {
                    refutation.reject().map_err(|e| self.error(node, e))?;
                    return Err(self.error(node, "selected call permission is explicitly refuted"));
                }
                Selection::Deferred => {}
                Selection::Absent => {
                    return Err(self.error(node, "selected call permission changed"));
                }
            }
            self.state.call_types.obligations.push((node, goal));
            return Ok(Some((kind, tuple)));
        }
        Ok(None)
    }

    fn possible_call_kind(
        &mut self,
        node: NodeId,
        subject: TermId,
        item: ItemId,
        assumptions: &crate::sema::traits::assumptions::Assumptions,
    ) -> Result<bool, Error> {
        let environment = self.trait_query_environment(node)?;
        let mut probe = Context::default();
        let subject = probe
            .import(&self.facts.types, &mut self.state.infer, subject)
            .map_err(|e| self.error(node, e))?;
        let tuple = probe
            .variable(Domain::Any)
            .map_err(|e| self.error(node, e))?;
        let goal = Predicate::trait_application_with_effect(
            subject,
            item,
            vec![Argument::Type(tuple)],
            self.invocation_effect()?,
        );
        let first_region = self.trait_region_start(node)?;
        let selected = selection::select_available(
            self.facts,
            &mut probe,
            &goal,
            Caller {
                assumptions,
                environment: environment.as_ref(),
                first_region,
                retain_sized: false,
                nesting: crate::sema::traits::selection::Nesting::ROOT,
                placeholders: None,
            },
            self.availability,
        );
        self.state
            .infer
            .account_work(probe.steps)
            .map_err(|e| self.error(node, e))?;
        Ok(!matches!(
            selected.map_err(|e| self.error(node, e))?,
            Selection::Absent
        ))
    }

    fn invocation_inputs(
        &mut self,
        node: NodeId,
        subject: TermId,
        tuple: TermId,
        arguments: List,
    ) -> Result<TermId, Error> {
        if self.head(node, tuple)?.is_none() {
            let count = self
                .report
                .package
                .unit(self.state.file)
                .configured
                .ast
                .list(arguments)
                .iter()
                .filter(|&&node| self.active(node))
                .count();
            self.state
                .infer
                .account_work(count)
                .map_err(|e| self.error(node, e))?;
            let mut fields = Vec::with_capacity(count);
            for _ in 0..count {
                fields.push(self.variable(node, Domain::Any)?);
            }
            let inferred = self
                .state
                .infer
                .construct(Constructor::Tuple, &fields)
                .map_err(|e| self.error(node, e))?;
            self.equate(node, tuple, inferred)?;
        }
        if self.head(node, tuple)? != Some(Constructor::Tuple) {
            return Err(self.error(node, "call trait arguments are not a tuple"));
        }
        let count = self
            .state
            .infer
            .arity(&self.facts.types, tuple)
            .map_err(|e| self.error(node, e))?
            .ok_or_else(|| self.error(node, "missing call tuple arity"))?;
        let mut fields = Vec::with_capacity(count + 1);
        for index in 0..count {
            fields.push(self.child(node, tuple, index)?);
        }
        let output = self
            .facts
            .associated
            .callable_output()
            .ok_or_else(|| self.error(node, "missing call Output"))?;
        let result = self
            .state
            .infer
            .projected(
                &self.facts.types,
                ItemId(output.0),
                subject,
                &[Argument::Type(tuple)],
                &[],
            )
            .map_err(|e| self.error(node, e))?;
        self.defer_normalization(node, crate::types::lifetime_proof::ScopeId::ROOT, result)?;
        fields.push(result);
        let parameters =
            u32::try_from(count).map_err(|_| self.error(node, "too many call arguments"))?;
        self.state
            .infer
            .construct(
                Constructor::Function {
                    parameters,
                    bound_regions: 0,
                    unsafe_: false,
                    convention: crate::types::callable::Convention::RUST,
                },
                &fields,
            )
            .map_err(|e| self.error(node, e))
    }

    pub(super) fn retain_invocation(
        &mut self,
        node: NodeId,
        source: NodeId,
        subject: TermId,
        dereferences: u32,
        kind: Option<CallTrait>,
    ) -> Result<(), Error> {
        if kind.is_some()
            && self.callee_use_effect()? != crate::support::constness::Constness::Ordinary
            && let Some(owner) = self.state.environment_owner
            && self
                .facts
                .function_indices
                .get(&owner)
                .and_then(|&index| self.facts.functions.get(index))
                .is_some_and(|function| function.body == Some(self.state.body))
        {
            // The defining body needs the same formal order as a demanded
            // function value, even when no client instantiates this function.
            crate::sema::declarations::prepare_callable_template(
                self.report,
                self.facts,
                self.key(node),
                owner,
            )?;
        }
        let borrow = match kind {
            Some(CallTrait::Fn | CallTrait::FnMut) => {
                let mutable = kind == Some(CallTrait::FnMut);
                let region = self.body_region(node, BodyRegionOrigin::ExpressionBorrow)?;
                let term = self
                    .state
                    .infer
                    .construct(Constructor::Reference { mutable, region }, &[subject])
                    .map_err(|e| self.error(node, e))?;
                if dereferences == 0 && self.invocation_plan(node).is_none() {
                    self.state.alignment.borrows.push((node, source));
                }
                Some(term)
            }
            _ => None,
        };
        self.state.call_types.field_receivers.push(subject);
        self.state.call_types.invocations.push((
            node,
            Pending {
                kind,
                dereferences,
                borrow,
                receiver: borrow.unwrap_or(subject),
            },
        ));
        Ok(())
    }

    pub(super) fn publish_invocations(&mut self, solved: &Solved) {
        for (node, pending) in &self.state.call_types.invocations {
            self.facts.invocations.insert(
                self.key(*node),
                Invocation {
                    kind: pending.kind,
                    dereferences: pending.dereferences,
                    borrow: pending.borrow.map(|term| solved.types[term.0 as usize]),
                    receiver: solved.types[pending.receiver.0 as usize],
                },
            );
        }
    }
}
