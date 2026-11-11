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
    Field(super::access_paths::OpenAccess),
    ReceiverDiscovery(super::access_paths::OpenAccess),
    Receiver {
        access: super::access_paths::OpenAccess,
        call: OpenCall,
    },
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
    Iteration {
        iterator: NodeId,
        pattern: NodeId,
        body: NodeId,
        into_iter: OpenCall,
        next: OpenCall,
        iterator_type: TermId,
        item: TermId,
        option: TermId,
        some: crate::sema::members::VariantId,
        none: crate::sema::members::VariantId,
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
impl OpenKind {
    pub(super) fn access(self) -> Option<super::access_paths::OpenAccess> {
        match self {
            Self::Field(access)
            | Self::ReceiverDiscovery(access)
            | Self::Receiver { access, .. } => Some(access),
            _ => None,
        }
    }
    pub(super) fn access_mut(&mut self) -> Option<&mut super::access_paths::OpenAccess> {
        match self {
            Self::Field(access)
            | Self::ReceiverDiscovery(access)
            | Self::Receiver { access, .. } => Some(access),
            _ => None,
        }
    }
    pub(super) fn call_count(self) -> Result<usize, &'static str> {
        match self {
            Self::Block { .. } => Ok(1),
            Self::Branch { .. } | Self::Iteration { .. } => Ok(2),
            Self::ReceiverDiscovery(_) => Err("method receiver discovery is unfinished"),
            Self::Receiver { access, .. } => access
                .steps
                .len()?
                .checked_add(1)
                .ok_or("method receiver call count overflow"),
            Self::Field(field) => {
                let count = field.steps.len()?;
                if count == 0 || field.field.is_none() || field.calls.is_none() {
                    return Err("field calls lack their completed actual discovery and selection");
                }
                Ok(count)
            }
        }
    }
    pub(super) fn solved(
        self,
        solved: &Solved,
    ) -> Result<crate::sema::protocol_plans::Kind, &'static str> {
        use crate::sema::protocol_plans::Kind;
        let ty = |term: TermId| {
            solved
                .types
                .get(term.0 as usize)
                .copied()
                .ok_or("source protocol lost a solved type")
        };
        Ok(match self {
            Self::Field(_) | Self::ReceiverDiscovery(_) | Self::Receiver { .. } => {
                return Err(
                    "field plan requires its actual final calls and checked step publication",
                );
            }
            Self::Block { body, call } => Kind::Block {
                body,
                from_output: call.solved(solved)?,
            },
            Self::Branch {
                operand,
                branch,
                residual,
                target,
                control_flow,
                continue_,
                break_,
            } => Kind::Branch {
                operand,
                branch: branch.solved(solved)?,
                residual: residual.solved(solved)?,
                target,
                control_flow: ty(control_flow)?,
                continue_,
                break_,
            },
            Self::Iteration {
                iterator,
                pattern,
                body,
                into_iter,
                next,
                iterator_type,
                item,
                option,
                some,
                none,
            } => Kind::Iteration {
                iterator,
                pattern,
                body,
                plan: crate::sema::iteration::ForPlan {
                    into_iter: into_iter.solved(solved)?,
                    next: next.solved(solved)?,
                    iterator: ty(iterator_type)?,
                    item: ty(item)?,
                    option: ty(option)?,
                    some,
                    none,
                },
            },
        })
    }
}
impl Checker<'_> {
    pub(super) fn protocol_open_call(
        &self,
        plan: OpenPlan,
        index: usize,
    ) -> Result<OpenCall, &'static str> {
        match (plan.kind, index) {
            (
                OpenKind::Block { call, .. }
                | OpenKind::Branch { branch: call, .. }
                | OpenKind::Iteration {
                    into_iter: call, ..
                },
                0,
            )
            | (
                OpenKind::Branch { residual: call, .. } | OpenKind::Iteration { next: call, .. },
                1,
            ) => Ok(call),
            (kind, index)
                if kind
                    .access()
                    .is_some_and(|access| access.steps.len().is_ok_and(|count| index < count)) =>
            {
                let field = kind.access().ok_or("access call lost its actual plan")?;
                let range = field.calls.ok_or("field call selection is unfinished")?;
                let (source, method) = self
                    .state
                    .call_types
                    .methods
                    .get(
                        (range.method as usize)
                            .checked_add(index)
                            .ok_or("field method range overflow")?,
                    )
                    .ok_or("field method range is outside its actual owner")?;
                let (call_source, signature) = self
                    .state
                    .calls
                    .get(
                        (range.call as usize)
                            .checked_add(index)
                            .ok_or("field call range overflow")?,
                    )
                    .ok_or("field call range is outside its actual owner")?;
                if (*source, *call_source) != (plan.node, plan.node) {
                    return Err("field call range belongs to another actual source node");
                }
                Ok(OpenCall {
                    callable: method.callable,
                    signature: *signature,
                })
            }
            (OpenKind::Receiver { access, call }, index) if index == access.steps.len()? => {
                Ok(call)
            }
            _ => Err("protocol call cursor is outside its actual plan"),
        }
    }
    pub(super) fn protocol_call(
        &mut self,
        node: NodeId,
        method: super::DefId,
    ) -> Result<OpenCall, Error> {
        let callable = self.implicit_source_function(node, method)?;
        let signature = self.child(node, callable, 0)?;
        let signature = self.call_signature(node, signature)?;
        self.state.calls.push((node, signature));
        Ok(OpenCall {
            callable,
            signature,
        })
    }
    pub(super) fn retain_try(&mut self, plan: OpenPlan) -> Result<usize, Error> {
        let plans = &self.state.call_types.protocol_plans;
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
                .protocol_plans
                .try_reserve_exact(additional)
                .map_err(|_| self.error(plan.node, "Try staging allocation failed"))?;
            if self.state.call_types.protocol_plans.capacity() > capacity {
                return Err(
                    self.error(plan.node, "Try staging allocation exceeds prepaid capacity")
                );
            }
        }
        let index = self.state.call_types.protocol_plans.len();
        self.state.call_types.protocol_plans.push(plan);
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
            .and_then(|index| self.state.call_types.protocol_plans.get(index))
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
