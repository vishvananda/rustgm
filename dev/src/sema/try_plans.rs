//! Canonical protocol calls share the current semantic-body call context owner.
use super::{bodies::SourceCallUse, iteration::Call, members::VariantId};
use crate::{resolve::unit::NodeKey, syntax::ast::NodeId, types::TypeId};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
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
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Plan {
    pub node: NodeKey,
    pub source_use: SourceCallUse,
    pub kind: Kind,
}
impl Plan {
    pub(crate) fn calls(self) -> [Option<Call>; 2] {
        match self.kind {
            Kind::Block { from_output, .. } => [Some(from_output), None],
            Kind::Branch {
                branch, residual, ..
            } => [Some(branch), Some(residual)],
        }
    }
}

/// Reauthenticate at the real CFG boundary, after canonical call publication.
pub(crate) fn checked(
    report: &crate::resolve::expansion::Report,
    facts: &super::model::Declarations,
    owner: NodeKey,
    node: NodeKey,
    id: super::bodies::TryPlanId,
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
    let protocol = super::compiler_items::try_protocol::source(report, facts, node)?;
    let actual = report
        .package
        .unit(node.file)
        .configured
        .ast
        .node(node.node)
        .kind;
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
    for (call, expected) in plan.calls().into_iter().zip(expected) {
        if let Some(call) = call
            && !matches!(facts.types.kind(call.callable),Some(Type::FunctionItem {item,signature,..})
                if Some(crate::resolve::model::DefId(item.0))==expected && signature==call.signature)
        {
            return Err(error(
                "Try plan changed its actual callable application or signature",
            ));
        }
    }
    Ok(plan)
}
