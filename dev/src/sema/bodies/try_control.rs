//! Try inference retains ordinary calls and the nearest lexical residual target.
use super::{Checker, Error, Frame, FrameKind, NodeId, TermId, Work};
use crate::{sema::compiler_items::try_protocol, types::infer::Solved};

#[derive(Clone, Copy)]
pub(super) struct OpenCall {
    pub callable: TermId,
    pub signature: TermId,
}
#[derive(Clone, Copy)]
pub(super) enum OpenKind {
    Block {
        body: NodeId,
        call: OpenCall,
    },
    Branch {
        operand: NodeId,
        branch: OpenCall,
        residual: OpenCall,
        target: Option<NodeId>,
        control_flow: TermId,
        continue_: crate::sema::members::VariantId,
        break_: crate::sema::members::VariantId,
    },
}
#[derive(Clone, Copy)]
pub(super) struct OpenPlan {
    pub node: NodeId,
    pub kind: OpenKind,
}
impl OpenCall {
    pub(super) fn solved(
        self,
        solved: &Solved,
    ) -> Result<crate::sema::iteration::Call, &'static str> {
        Ok(crate::sema::iteration::Call {
            callable: *solved
                .types
                .get(self.callable.0 as usize)
                .ok_or("Try call lost its solved callable")?,
            signature: *solved
                .types
                .get(self.signature.0 as usize)
                .ok_or("Try call lost its solved signature")?,
        })
    }
}
impl Checker<'_> {
    fn protocol_call(&mut self, node: NodeId, method: super::DefId) -> Result<OpenCall, Error> {
        let callable = self.implicit_source_function(node, method)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        self.state.calls.push((node, signature));
        Ok(OpenCall {
            callable,
            signature,
        })
    }
    fn retain_try(&mut self, plan: OpenPlan) -> Result<usize, Error> {
        let plans = &self.state.call_types.try_plans;
        let growing = plans.len() == plans.capacity();
        let capacity = plans.len().saturating_mul(2).max(4);
        let bytes = if growing {
            capacity
                .checked_add(plans.capacity())
                .and_then(|count| count.checked_mul(size_of::<OpenPlan>()))
                .ok_or_else(|| self.error(plan.node, "Try staging storage overflow"))?
        } else {
            0
        };
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            1 + usize::from(growing) * plans.len(),
            bytes,
        )?;
        let additional = capacity - plans.len();
        if growing {
            self.state
                .call_types
                .try_plans
                .try_reserve_exact(additional)
                .map_err(|_| self.error(plan.node, "Try staging allocation failed"))?;
            if self.state.call_types.try_plans.capacity() > capacity {
                return Err(
                    self.error(plan.node, "Try staging allocation exceeds prepaid capacity")
                );
            }
        }
        let index = self.state.call_types.try_plans.len();
        self.state.call_types.try_plans.push(plan);
        Ok(index)
    }
    pub(super) fn enter_try_block(
        &mut self,
        node: NodeId,
        body: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let protocol = try_protocol::source(self.report, self.facts, self.key(node))?;
        let call = self.protocol_call(node, protocol.from_output)?;
        let output = self.child(node, call.signature, 0)?;
        let result = self.child(node, call.signature, 1)?;
        if let Some(expected) = expected {
            self.shape_equation(node, result, expected)?;
        }
        let plan = self.retain_try(OpenPlan {
            node,
            kind: OpenKind::Block { body, call },
        })?;
        self.state.frames.push(Frame {
            node,
            body,
            kind: FrameKind::Try,
            label: None,
            result,
            breaks: Some(plan),
            expected,
            in_condition: false,
        });
        self.state.work.push(Work::Enter(body, Some(output)));
        Ok(())
    }
    pub(super) fn finish_try_block(&mut self, node: NodeId, body: NodeId) -> Result<TermId, Error> {
        let frame = self
            .state
            .frames
            .pop()
            .filter(|frame| {
                frame.node == node && frame.body == body && frame.kind == FrameKind::Try
            })
            .ok_or_else(|| self.error(node, "Try block lost its actual lexical frame"))?;
        let plan = frame
            .breaks
            .and_then(|index| self.state.call_types.try_plans.get(index))
            .ok_or_else(|| self.error(node, "Try block lost its staged from_output call"))?;
        let OpenKind::Block { call, body: actual } = plan.kind else {
            return Err(self.error(node, "Try block has a different staged operation"));
        };
        if actual != body {
            return Err(self.error(node, "Try block body changed during inference"));
        }
        let output = self.child(node, call.signature, 0)?;
        self.coerce(body, output)?;
        self.state.diverges[node.get() as usize] = self.state.diverges[body.get() as usize];
        Ok(frame.result)
    }
    pub(super) fn question_mark(&mut self, node: NodeId, operand: NodeId) -> Result<TermId, Error> {
        let protocol = try_protocol::source(self.report, self.facts, self.key(node))?;
        let branch = self.protocol_call(node, protocol.branch)?;
        let input = self.child(node, branch.signature, 0)?;
        self.coerce(operand, input)?;
        let control_flow = self.child(node, branch.signature, 1)?;
        let residual_type = self.child(node, control_flow, 0)?;
        let output = self.child(node, control_flow, 1)?;
        crate::sema::header_attributes::charge_source_contract_query(
            self.facts,
            self.state.frames.len(),
        )?;
        let target = self
            .state
            .frames
            .iter()
            .rev()
            .find(|frame| frame.kind == FrameKind::Try)
            .copied();
        let result = if let Some(frame) = target {
            frame.result
        } else {
            if !self.state.in_function {
                return Err(self.error(node, "question mark has no function or Try target"));
            }
            self.result_term(node)?
        };
        let method = if target.is_some() {
            protocol.into_try_type
        } else {
            protocol.from_residual
        };
        let residual = self.protocol_call(node, method)?;
        let residual_input = self.child(node, residual.signature, 0)?;
        let residual_result = self.child(node, residual.signature, 1)?;
        self.shape_equation(node, residual_type, residual_input)?;
        self.shape_equation(node, residual_result, result)?;
        self.retain_try(OpenPlan {
            node,
            kind: OpenKind::Branch {
                operand,
                branch,
                residual,
                target: target.map(|frame| frame.node),
                control_flow,
                continue_: protocol.continue_,
                break_: protocol.break_,
            },
        })?;
        Ok(output)
    }
}
