//! Operator receivers use the same explicit place, borrow and evaluation-order machinery.
use super::{Builder, Error, List, NodeId, NodeKey, Operation, Rvalue, ValueId, ValueList};
use crate::{sema::invocations::Invocation, support::call_traits::CallTrait};
impl Builder<'_> {
    pub(super) fn invocation(&self, node: NodeId) -> Option<Invocation> {
        self.facts
            .invocations
            .get(&NodeKey {
                node,
                ..self.body.source
            })
            .copied()
    }
    pub(super) fn invocation_receiver(
        &mut self,
        node: NodeId,
        callee: NodeId,
    ) -> Result<(), Error> {
        let place = self.pop_place(callee)?;
        let Some(place) = place.filter(|_| self.current.is_some()) else {
            return Ok(());
        };
        let selected = self
            .invocation(node)
            .ok_or_else(|| self.error(node, "missing call operator selection"))?;
        let value = self.receiver_value(
            node,
            place,
            self.remaining_receiver_dereferences(
                node,
                crate::sema::closures::AccessKind::Invocation,
                selected.dereferences,
            )?,
            selected.borrow,
            selected.receiver,
            crate::body::ir::BorrowKind::TwoPhase,
        )?;
        self.scratch.receivers.insert(node, value);
        Ok(())
    }
    pub(super) fn invoke_operator(
        &mut self,
        node: NodeId,
        kind: CallTrait,
        receiver: ValueId,
        arguments: List,
    ) -> Result<ValueId, Error> {
        let Some(Operation::Call { signature }) = self.fact(node)?.operation else {
            return Err(self.error(node, "missing call signature"));
        };
        let arguments = self.call_inputs(node, arguments, Some(receiver))?;
        self.evaluate(
            node,
            self.ty(node)?,
            Rvalue::Invoke {
                kind,
                signature,
                arguments,
            },
        )
    }
    pub(super) fn call_inputs(
        &mut self,
        node: NodeId,
        arguments: List,
        receiver: Option<ValueId>,
    ) -> Result<ValueList, Error> {
        let start = self.index(node, self.body.arguments.len())?;
        self.body.arguments.extend(receiver);
        for &argument in self
            .report
            .package
            .unit(self.body.source.file)
            .configured
            .ast
            .list(arguments)
        {
            if self.active(argument) {
                self.body.arguments.push(self.value(argument)?);
            }
        }
        let end = self.index(node, self.body.arguments.len())?;
        Ok(ValueList { start, end })
    }
}
