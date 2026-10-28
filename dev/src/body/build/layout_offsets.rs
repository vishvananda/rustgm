//! Real builtin offsets emit the authenticated ordinary intrinsic and operands.
use super::{Builder, Error, NodeId, Operation, Rvalue, ValueId};
use crate::{
    body::ir::{ScalarId, ValueList},
    sema::model::Scalar,
    types::{Kind, Primitive, TypeId},
};
impl Builder<'_> {
    pub(super) fn offset_value(&mut self, node: NodeId) -> Result<ValueId, Error> {
        let Some(Operation::OffsetOf { plan }) = self.fact(node)?.operation else {
            return Err(self.error(node, "offset expression lost its checked member plan"));
        };
        let plan = crate::sema::displacement_plans::checked(
            self.report,
            self.facts,
            self.source,
            crate::resolve::unit::NodeKey {
                node,
                ..self.source
            },
            plan,
        )?;
        let identity = self
            .facts
            .members
            .identity(plan.field)
            .ok_or_else(|| self.error(node, "offset field lost its storage coordinates"))?;
        let Some(Kind::FunctionItem { item, .. }) = self.facts.types.kind(plan.call.callable)
        else {
            return Err(self.error(node, "offset lost its actual intrinsic item"));
        };
        let callee = self.evaluate(
            node,
            plan.call.callable,
            Rvalue::Function(crate::resolve::model::DefId(item.0)),
        )?;
        let mut values = [callee; 2];
        for (slot, operand) in values.iter_mut().zip([identity.variant, identity.field]) {
            let scalar = ScalarId(self.index(node, self.body.scalars.len())?);
            self.body.scalars.push(Scalar::Integer(u128::from(operand)));
            *slot = self.evaluate(
                node,
                TypeId::primitive(Primitive::U32),
                Rvalue::Scalar(scalar),
            )?;
        }
        let start = self.index(node, self.body.arguments.len())?;
        self.body.arguments.extend(values);
        let end = self.index(node, self.body.arguments.len())?;
        self.evaluate(
            node,
            self.ty(node)?,
            Rvalue::Call {
                signature: plan.call.signature,
                callee,
                arguments: ValueList { start, end },
            },
        )
    }
}
