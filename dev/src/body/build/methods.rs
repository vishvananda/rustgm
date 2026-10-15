//! Receiver places are evaluated once, adjusted, then passed before explicit arguments.
use super::{Builder, Error, List, NodeId, NodeKey, Rvalue, Type, ValueId};
use crate::body::ir::BorrowKind;

impl Builder<'_> {
    pub(super) fn method_receiver(&mut self, node: NodeId) -> Result<(), Error> {
        let key = NodeKey {
            node,
            ..self.body.source
        };
        let site = self
            .report
            .resolution
            .method_calls
            .get(&key)
            .ok_or_else(|| self.error(node, "missing method source operands"))?;
        let place = self.pop_place(site.receiver)?;
        let Some(mut place) = place.filter(|_| self.current.is_some()) else {
            return Ok(());
        };
        let selected = self
            .facts
            .methods
            .calls
            .get(&key)
            .copied()
            .ok_or_else(|| self.error(node, "missing selected receiver conversion"))?;
        for _ in 0..selected.dereferences {
            place = self.dereference_place(node, place)?;
        }
        let mut value = if let Some(ty) = selected.borrow {
            let Some(Type::Reference {
                referent, mutable, ..
            }) = self.facts.types.kind(ty)
            else {
                return Err(self.error(node, "method borrow is not a reference"));
            };
            if referent != self.place_type(node, place)? {
                return Err(self.error(node, "method borrow differs from selected receiver place"));
            }
            let kind = if mutable {
                BorrowKind::TwoPhase
            } else {
                BorrowKind::Immediate
            };
            self.evaluate(node, ty, Rvalue::Borrow { place, kind })?
        } else {
            let ty = self.place_type(node, place)?;
            self.evaluate(node, ty, Rvalue::Read(place))?
        };
        if self.body.values[value.0 as usize].ty != selected.receiver {
            value = self.evaluate(node, selected.receiver, Rvalue::Constrain(value))?;
        }
        self.scratch.method_receivers.insert(node, value);
        Ok(())
    }

    pub(super) fn method_call(&mut self, node: NodeId, arguments: List) -> Result<ValueId, Error> {
        let key = NodeKey {
            node,
            ..self.body.source
        };
        let selected = self
            .facts
            .methods
            .calls
            .get(&key)
            .ok_or_else(|| self.error(node, "missing selected method callable"))?;
        let callee = self.evaluate(node, selected.callable, Rvalue::Function(selected.method))?;
        let receiver = self
            .scratch
            .method_receivers
            .remove(&node)
            .ok_or_else(|| self.error(node, "missing evaluated method receiver"))?;
        self.invoke(node, callee, arguments, Some(receiver))
    }
}
