//! Receiver places are evaluated once, adjusted, then passed before explicit arguments.
use super::{Builder, Error, List, NodeId, NodeKey, Rvalue, Type, ValueId};
use crate::body::ir::BorrowKind;

impl Builder<'_> {
    pub(super) fn receiver_call_signature(
        &self,
        node: NodeId,
        callee: ValueId,
    ) -> Result<crate::types::TypeId, Error> {
        match self.fact(node)?.operation {
            Some(crate::sema::model::Operation::MethodReceiver { plan }) => {
                let key = NodeKey {
                    node,
                    ..self.source
                };
                let plan = crate::sema::protocol_plans::checked(
                    self.report,
                    self.facts,
                    self.source,
                    key,
                    plan,
                )?;
                match plan.kind {
                    crate::sema::protocol_plans::Kind::Receiver { call, .. }
                        if self
                            .body
                            .values
                            .get(callee.0 as usize)
                            .is_some_and(|value| value.ty == call.callable) =>
                    {
                        Ok(call.signature)
                    }
                    _ => Err(self.error(node, "method receiver changed its actual final call")),
                }
            }
            _ => Err(self.error(node, "missing call signature")),
        }
    }

    pub(super) fn method_receiver(&mut self, node: NodeId) -> Result<(), Error> {
        let key = NodeKey {
            node,
            ..self.source
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
        let access = if let Some(crate::sema::model::Operation::MethodReceiver { plan }) =
            self.fact(node)?.operation
        {
            let plan = crate::sema::protocol_plans::checked(
                self.report,
                self.facts,
                self.source,
                key,
                plan,
            )?;
            place = self.access_call_place(
                node,
                place,
                plan,
                crate::sema::closures::AccessKind::Method,
            )?;
            true
        } else {
            false
        };
        let dereferences = if access {
            selected.dereferences
        } else {
            self.remaining_receiver_dereferences(
                node,
                crate::sema::closures::AccessKind::Method,
                selected.dereferences,
            )?
        };
        let value = self.receiver_value(
            node,
            place,
            dereferences,
            selected.borrow,
            selected.receiver,
            BorrowKind::TwoPhase,
        )?;
        self.scratch.receivers.insert(node, value);
        Ok(())
    }

    pub(super) fn receiver_value(
        &mut self,
        node: NodeId,
        mut place: crate::body::ir::Place,
        dereferences: u32,
        borrow: Option<crate::types::TypeId>,
        receiver: crate::types::TypeId,
        mutable_kind: BorrowKind,
    ) -> Result<ValueId, Error> {
        for _ in 0..dereferences {
            place = self.dereference_place(node, place)?;
        }
        let mut value = if let Some(ty) = borrow {
            let Some(Type::Reference {
                referent, mutable, ..
            }) = self.facts.types.kind(ty)
            else {
                return Err(self.error(node, "method borrow is not a reference"));
            };
            if referent != self.place_type(node, place)? {
                let actual = self.place_type(node, place)?;
                return Err(self.error(node, format!(
                    "method borrow differs from selected receiver place: actual {actual:?} {:?}, expected {referent:?} {:?}, place {place:?}",
                    self.facts.types.kind(actual), self.facts.types.kind(referent),
                )));
            }
            let kind = if mutable {
                mutable_kind
            } else {
                BorrowKind::Immediate
            };
            self.evaluate(node, ty, Rvalue::Borrow { place, kind })?
        } else {
            let ty = self.place_type(node, place)?;
            self.evaluate(node, ty, Rvalue::Read(place))?
        };
        if self.body.values[value.0 as usize].ty != receiver {
            value = self.evaluate(node, receiver, Rvalue::Constrain(value))?;
        }
        Ok(value)
    }

    pub(super) fn method_call(&mut self, node: NodeId, arguments: List) -> Result<ValueId, Error> {
        let key = NodeKey {
            node,
            ..self.source
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
            .receivers
            .remove(&node)
            .ok_or_else(|| self.error(node, "missing evaluated method receiver"))?;
        self.invoke(node, callee, arguments, Some(receiver))
    }
}
