//! A retained Field chain evaluates its original operand and each actual call once.
use super::{Builder, Error, NodeId, NodeKey, Place, Rvalue, Type};
use crate::{
    body::ir::{BorrowKind, ProjectionKind},
    sema::{
        bodies::ProtocolPlanId,
        protocol_plans::{self, Kind},
    },
};

impl Builder<'_> {
    pub(super) fn field_call_place(
        &mut self,
        node: NodeId,
        mut place: Place,
        id: ProtocolPlanId,
    ) -> Result<Place, Error> {
        let key = NodeKey {
            node,
            ..self.source
        };
        let plan = protocol_plans::checked(self.report, self.facts, self.source, key, id)?;
        let Kind::Field {
            field,
            receiver,
            dereferences,
            steps,
            ..
        } = plan.kind
        else {
            return Err(self.error(node, "field construction received another actual protocol"));
        };
        let mut calls = plan.calls().map_err(|e| self.error(node, e))?;
        while let Some(step) =
            protocol_plans::access::next(self.facts, plan, self.source, steps, &mut calls)
                .map_err(|e| self.error(node, e))?
        {
            let Some(Type::Function { parameters, .. }) =
                self.facts.types.kind(step.call.signature)
            else {
                return Err(self.error(node, "field construction lost its actual call signature"));
            };
            let Some([expected]) = self.facts.types.list(parameters) else {
                return Err(
                    self.error(node, "field construction changed its actual receiver arity")
                );
            };
            let argument = self.receiver_value(
                node,
                place,
                step.dereferences,
                Some(step.borrow),
                *expected,
                BorrowKind::Immediate,
            )?;
            let mut reference = self.implicit_call(node, step.call, argument)?;
            if self.body.values[reference.0 as usize].ty != step.output_view {
                reference = self.evaluate(node, step.output_view, Rvalue::Constrain(reference))?;
            }
            let stored = self.materialize_place(node, reference)?.ok_or_else(|| {
                self.error(
                    node,
                    "field call lost its actual returned reference storage",
                )
            })?;
            place = self.dereference_place(node, stored)?;
        }
        for _ in 0..dereferences {
            place = self.dereference_place(node, place)?;
        }
        if self.place_type(node, place)? != receiver {
            return Err(self.error(node, "field construction changed its full final receiver"));
        }
        self.project(node, self.ty(node)?, place, ProjectionKind::Field(field))
    }
}
