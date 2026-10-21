//! Unary syntax consumes its selected scalar, reference or raw-address operation.
use super::{Builder, Error, NodeId, Operation, Rvalue, ValueId};
use crate::sema::model::Cast;

impl Builder<'_> {
    pub(super) fn unary_value(&mut self, node: NodeId, operand: NodeId) -> Result<ValueId, Error> {
        if let Some(value) = self.language_call(node, &[operand])? {
            return Ok(value);
        }
        let kind = match self.fact(node)?.operation {
            Some(Operation::Borrow) => return self.finish_borrow(node),
            Some(Operation::Address) => return self.finish_address(node),
            Some(Operation::PointerAddress { operand }) => Rvalue::Cast {
                kind: Cast::Pointer,
                operand: self.value(operand)?,
            },
            Some(Operation::Unary {
                operator, literal, ..
            }) => Rvalue::Unary {
                operator,
                operand: self.value(operand)?,
                literal,
            },
            _ => return Err(self.error(node, "missing selected unary operation")),
        };
        self.evaluate(node, self.ty(node)?, kind)
    }
}

impl Builder<'_> {
    pub(super) fn language_call(
        &mut self,
        node: NodeId,
        operands: &[NodeId],
    ) -> Result<Option<ValueId>, Error> {
        let key = crate::resolve::unit::NodeKey {
            node,
            ..self.body.source
        };
        let Some(selected) = self.facts.methods.calls.get(&key).copied() else {
            return Ok(None);
        };
        let Some(Operation::Call { signature }) = self.fact(node)?.operation else {
            return Err(self.error(node, "missing library operator signature"));
        };
        let callee = self.evaluate(node, selected.callable, Rvalue::Function(selected.method))?;
        let start = self.index(node, self.body.arguments.len())?;
        for &operand in operands {
            self.body.arguments.push(self.value(operand)?);
        }
        let end = self.index(node, self.body.arguments.len())?;
        let arguments = crate::body::ir::ValueList { start, end };
        self.evaluate(
            node,
            self.ty(node)?,
            Rvalue::Call {
                signature,
                callee,
                arguments,
            },
        )
        .map(Some)
    }
}

impl Builder<'_> {
    pub(super) fn language_index_receiver(&mut self, node: NodeId) -> Result<(), Error> {
        let key = crate::resolve::unit::NodeKey {
            node,
            ..self.body.source
        };
        let selected = self
            .facts
            .methods
            .calls
            .get(&key)
            .copied()
            .ok_or_else(|| self.error(node, "missing library index selection"))?;
        let base = self.scratch.places.last().copied().flatten();
        if let Some(base) = base.filter(|_| self.current.is_some()) {
            let base = self.automatic_dereferences(node, base)?;
            let receiver = self.receiver_value(
                node,
                base,
                0,
                selected.borrow,
                selected.receiver,
                crate::body::ir::BorrowKind::Immediate,
            )?;
            self.scratch.receivers.insert(node, receiver);
        }
        Ok(())
    }
    pub(super) fn language_index_place(
        &mut self,
        node: NodeId,
    ) -> Result<crate::body::ir::Place, Error> {
        let key = crate::resolve::unit::NodeKey {
            node,
            ..self.body.source
        };
        let selected = self
            .facts
            .methods
            .calls
            .get(&key)
            .copied()
            .ok_or_else(|| self.error(node, "missing library index selection"))?;
        let Some(Operation::Call { signature }) = self.fact(node)?.operation else {
            return Err(self.error(node, "missing library index signature"));
        };
        let super::Kind::Index { index, .. } = self.kind(node) else {
            return Err(self.error(node, "invalid library index syntax"));
        };
        let receiver = self
            .scratch
            .receivers
            .remove(&node)
            .ok_or_else(|| self.error(node, "missing evaluated index receiver"))?;
        let callee = self.evaluate(node, selected.callable, Rvalue::Function(selected.method))?;
        let start = self.index(node, self.body.arguments.len())?;
        let index = self.value(index)?;
        self.body.arguments.extend([receiver, index]);
        let end = self.index(node, self.body.arguments.len())?;
        let arguments = crate::body::ir::ValueList { start, end };
        let Some(crate::types::Kind::Function { result, .. }) = self.facts.types.kind(signature)
        else {
            return Err(self.error(node, "index method signature is not a function"));
        };
        let reference = self.evaluate(
            node,
            result,
            Rvalue::Call {
                signature,
                callee,
                arguments,
            },
        )?;
        let place = self
            .materialize_place(node, reference)?
            .ok_or_else(|| self.error(node, "missing index reference storage"))?;
        self.dereference_place(node, place)
    }
}
