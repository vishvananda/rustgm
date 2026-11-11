//! Canonical protocol calls share the current semantic-body call context owner.
use super::{bodies::SourceCallUse, iteration::Call, members::VariantId};
use crate::{resolve::unit::NodeKey, syntax::ast::NodeId, types::TypeId};
pub(crate) mod access;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    Field {
        operand: NodeId,
        field: super::members::Selection,
        receiver: TypeId,
        dereferences: u32,
        steps: access::Range,
    },
    Coercion {
        operand: NodeId,
        receiver: TypeId,
        dereferences: u32,
        steps: access::Range,
    },
    Receiver {
        operand: NodeId,
        receiver: TypeId,
        dereferences: u32,
        steps: access::Range,
        call: Call,
    },
    Block {
        body: NodeId,
        from_output: Call,
    },
    Branch {
        operand: NodeId,
        branch: Call,
        residual: Call,
        target: Option<NodeId>,
        control_flow: TypeId,
        continue_: VariantId,
        break_: VariantId,
    },
    Iteration {
        iterator: NodeId,
        pattern: NodeId,
        body: NodeId,
        plan: super::iteration::ForPlan,
    },
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Plan {
    pub node: NodeKey,
    pub source_use: SourceCallUse,
    pub kind: Kind,
}
impl Kind {
    pub(crate) fn retained_id(
        self,
        facts: &super::model::Declarations,
        node: NodeKey,
    ) -> Option<super::bodies::ProtocolPlanId> {
        if matches!(self, Self::Coercion { .. }) {
            return facts.reference_adjustments.get(&node)?.plan;
        }
        match facts.node(node)?.operation? {
            super::model::Operation::Field { plan, .. } => plan,
            super::model::Operation::MethodReceiver { plan }
            | super::model::Operation::Try { plan }
            | super::model::Operation::Iteration { plan } => Some(plan),
            _ => None,
        }
    }

    pub(crate) fn access(self) -> Option<(NodeId, TypeId, u32, access::Range)> {
        match self {
            Self::Field {
                operand,
                receiver,
                dereferences,
                steps,
                ..
            }
            | Self::Coercion {
                operand,
                receiver,
                dereferences,
                steps,
            }
            | Self::Receiver {
                operand,
                receiver,
                dereferences,
                steps,
                ..
            } => Some((operand, receiver, dereferences, steps)),
            _ => None,
        }
    }
}
impl Plan {
    pub(crate) fn calls(self) -> Result<Calls, &'static str> {
        let (next, remaining) = match self.kind {
            Kind::Field { steps, .. }
            | Kind::Coercion { steps, .. }
            | Kind::Receiver { steps, .. } => {
                let count = steps
                    .end
                    .checked_sub(steps.start)
                    .filter(|&count| count != 0)
                    .ok_or("field calls have an invalid actual step range")?;
                (
                    steps.first.get() - 1,
                    count
                        .checked_add(u32::from(matches!(self.kind, Kind::Receiver { .. })))
                        .ok_or("receiver call count overflow")?,
                )
            }
            Kind::Block { .. } => (0, 1),
            Kind::Branch { .. } | Kind::Iteration { .. } => (0, 2),
        };
        Ok(Calls { next, remaining })
    }
    pub(crate) fn next_call(
        self,
        facts: &super::model::Declarations,
        owner: NodeKey,
        calls: &mut Calls,
    ) -> Result<Option<Call>, &'static str> {
        if calls.remaining == 0 {
            return Ok(None);
        }
        let call = match (self.kind, calls.next) {
            (Kind::Receiver { call, .. }, 0) if calls.remaining == 1 => {
                calls.remaining = 0;
                return Ok(Some(call));
            }
            (
                Kind::Field { steps, .. }
                | Kind::Coercion { steps, .. }
                | Kind::Receiver { steps, .. },
                _,
            ) => {
                return access::next(facts, self, owner, steps, calls)
                    .map(|step| step.map(|step| step.call));
            }
            (
                Kind::Block {
                    from_output: call, ..
                }
                | Kind::Branch { branch: call, .. },
                0,
            )
            | (Kind::Branch { residual: call, .. }, 1) => {
                calls.next += 1;
                call
            }
            (Kind::Iteration { plan, .. }, index @ 0..=1) => {
                calls.next += 1;
                if index == 0 {
                    plan.into_iter
                } else {
                    plan.next
                }
            }
            _ => return Err("protocol call cursor exceeds its actual source plan"),
        };
        calls.remaining -= 1;
        Ok(Some(call))
    }
}

/// A bounded traversal position; it grants no call or body permission.
pub(crate) struct Calls {
    next: u32,
    remaining: u32,
}
const _: [(); 64] = [(); size_of::<Plan>()];
const _: [(); 8] = [(); size_of::<Calls>()];

/// Reauthenticate at the real CFG boundary, after canonical call publication.
pub(crate) fn checked(
    report: &crate::resolve::expansion::Report,
    facts: &super::model::Declarations,
    owner: NodeKey,
    node: NodeKey,
    id: super::bodies::ProtocolPlanId,
) -> Result<Plan, crate::resolve::package::Error> {
    use crate::{
        syntax::ast::{Kind as Syntax, Unary},
        types::Kind as Type,
    };
    let error = |message| super::model::Declarations::error(report, node, message);
    super::header_attributes::charge_source_contract_query(facts, 5)?;
    let plan = id.current(facts, owner, node).map_err(error)?;
    if matches!(plan.kind, Kind::Block { .. }) {
        crate::resolve::require_try_block(report, node)?;
    }
    let actual = report
        .package
        .unit(node.file)
        .configured
        .ast
        .node(node.node)
        .kind;
    if plan.kind.access().is_some() {
        access::check(report, facts, owner, plan)?;
        return Ok(plan);
    }
    if let Kind::Iteration {
        iterator,
        pattern,
        body,
        plan: iteration,
    } = plan.kind
    {
        if !matches!(actual, Syntax::For { iterator: a, pattern: b, body: c }
            if (a, b, c) == (iterator, pattern, body))
        {
            return Err(error("iteration plan changed its actual source operation"));
        }
        super::compiler_items::for_roles::check_plan(report, facts, node, iteration)?;
        return Ok(plan);
    }
    let protocol = super::compiler_items::try_protocol::source(report, facts, node)?;
    let expected = match (plan.kind, actual) {
        (Kind::Block { body, .. }, Syntax::TryBlock { body: actual, .. }) if body == actual => {
            [Some(protocol.from_output), None]
        }
        (
            Kind::Branch {
                operand,
                target,
                continue_,
                break_,
                ..
            },
            Syntax::Unary {
                op: Unary::Try,
                operand: actual,
            },
        ) if operand == actual && continue_ == protocol.continue_ && break_ == protocol.break_ => [
            Some(protocol.branch),
            Some(if target.is_some() {
                protocol.into_try_type
            } else {
                protocol.from_residual
            }),
        ],
        _ => {
            return Err(error(
                "Try plan changed its actual source operation or variants",
            ));
        }
    };
    let mut calls = plan.calls().map_err(error)?;
    let mut index = 0;
    while let Some(call) = plan.next_call(facts, owner, &mut calls).map_err(error)? {
        let expected = expected.get(index).copied().flatten();
        if !matches!(facts.types.kind(call.callable),Some(Type::FunctionItem {item,signature,..})
                if Some(crate::resolve::model::DefId(item.0))==expected && signature==call.signature)
        {
            return Err(error(
                "Try plan changed its actual callable application or signature",
            ));
        }
        index += 1;
    }
    Ok(plan)
}
