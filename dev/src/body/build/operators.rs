//! Unary syntax consumes its selected scalar, reference or raw-address operation.
use super::{Builder, Error, NodeId, Operation, Rvalue, ValueId};
use crate::sema::model::Cast;

impl Builder<'_> {
    pub(super) fn cast_value(
        &mut self,
        node: NodeId,
        operand: NodeId,
        ty: crate::types::TypeId,
        kind: Cast,
    ) -> Result<ValueId, Error> {
        let value = self.value(operand)?;
        let direct_variant = kind == Cast::EnumToInteger
            && self
                .facts
                .direct_variant(
                    self.report,
                    crate::resolve::unit::NodeKey {
                        node: operand,
                        ..self.source
                    },
                )
                .is_some();
        self.evaluate(
            node,
            ty,
            Rvalue::Cast {
                kind,
                operand: value,
                direct_variant,
            },
        )
    }
    pub(super) fn unary_value(&mut self, node: NodeId, operand: NodeId) -> Result<ValueId, Error> {
        if let Some(value) = self.language_call(node, &[operand])? {
            return Ok(value);
        }
        let kind = match self.fact(node)?.operation {
            Some(Operation::Borrow) => return self.finish_borrow(node),
            Some(Operation::Address) => return self.finish_address(node),
            Some(Operation::PointerAddress { operand }) => Rvalue::Cast {
                direct_variant: false,
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
            ..self.source
        };
        let Some(selected) = self.facts.methods.calls.get(&key).copied() else {
            return Ok(None);
        };
        let Some(Operation::Call { signature, .. }) = self.fact(node)?.operation else {
            return Err(self.error(node, "missing library operator signature"));
        };
        let callee = self.evaluate(node, selected.callable, Rvalue::Function(selected.method))?;
        let start = self.index(node, self.body.arguments.len())?;
        let borrowed = selected.borrow.is_some();
        if borrowed {
            let receiver = self
                .scratch
                .receivers
                .remove(&node)
                .ok_or_else(|| self.error(node, "missing evaluated operator receiver"))?;
            self.body.arguments.push(receiver);
        }
        if self.comparison_operator(node) {
            let right = self
                .scratch
                .comparison_arguments
                .as_deref_mut()
                .and_then(|arguments| arguments.remove(&node))
                .ok_or_else(|| self.error(node, "missing comparison RHS borrow"))?;
            self.body.arguments.push(right);
        } else {
            for &operand in &operands[usize::from(borrowed)..] {
                self.body.arguments.push(self.value(operand)?);
            }
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
    pub(super) fn comparison_operator(&self, node: NodeId) -> bool {
        self.facts
            .methods
            .calls
            .get(&crate::resolve::unit::NodeKey {
                node,
                ..self.source
            })
            .is_some_and(|selected| {
                crate::sema::compiler_items::comparison_member(
                    self.report,
                    self.facts,
                    selected.method,
                )
                .is_some()
            })
    }
    pub(super) fn comparison_argument(&mut self, node: NodeId) -> Result<(), Error> {
        let super::Kind::Binary { right, .. } = self.kind(node) else {
            return Err(self.error(node, "comparison requires binary syntax"));
        };
        let place = self.pop_place(right)?;
        let Some(place) = place.filter(|_| self.current.is_some()) else {
            return Ok(());
        };
        let Some(Operation::Call { signature, .. }) = self.fact(node)?.operation else {
            return Err(self.error(node, "comparison requires selected call"));
        };
        let Some(crate::types::Kind::Function { parameters, .. }) =
            self.facts.types.kind(signature)
        else {
            return Err(self.error(node, "comparison signature is not callable"));
        };
        let expected = *self
            .facts
            .types
            .list(parameters)
            .and_then(|p| p.get(1))
            .ok_or_else(|| self.error(node, "comparison has no RHS parameter"))?;
        let actual = self
            .facts
            .methods
            .comparison_borrow(crate::resolve::unit::NodeKey {
                node,
                ..self.source
            })
            .ok_or_else(|| self.error(node, "comparison RHS lacks physical shared-borrow type"))?;
        let value = self.receiver_value(
            node,
            place,
            0,
            Some(actual),
            expected,
            crate::body::ir::BorrowKind::Immediate,
        )?;
        self.scratch
            .comparison_arguments
            .get_or_insert_default()
            .insert(node, value);
        Ok(())
    }
    pub(super) fn borrowed_language_operator(&self, node: NodeId) -> bool {
        self.facts
            .methods
            .calls
            .get(&crate::resolve::unit::NodeKey {
                node,
                ..self.source
            })
            .is_some_and(|selected| selected.borrow.is_some())
    }
    pub(super) fn language_receiver(&mut self, node: NodeId) -> Result<(), Error> {
        if let super::Kind::Binary { left, .. } = self.kind(node) {
            let place = self.pop_place(left)?;
            let Some(place) = place.filter(|_| self.current.is_some()) else {
                return Ok(());
            };
            let key = crate::resolve::unit::NodeKey {
                node,
                ..self.source
            };
            let selected = self
                .facts
                .methods
                .calls
                .get(&key)
                .copied()
                .ok_or_else(|| self.error(node, "missing assignment operator selection"))?;
            let receiver = self.receiver_value(
                node,
                place,
                selected.dereferences,
                selected.borrow,
                selected.receiver,
                if self.comparison_operator(node) {
                    crate::body::ir::BorrowKind::Immediate
                } else {
                    crate::body::ir::BorrowKind::TwoPhase
                },
            )?;
            self.scratch.receivers.insert(node, receiver);
            Ok(())
        } else {
            self.language_index_receiver(node)
        }
    }
    pub(super) fn language_index_receiver(&mut self, node: NodeId) -> Result<(), Error> {
        let key = crate::resolve::unit::NodeKey {
            node,
            ..self.source
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
            ..self.source
        };
        let selected = self
            .facts
            .methods
            .calls
            .get(&key)
            .copied()
            .ok_or_else(|| self.error(node, "missing library index selection"))?;
        let Some(Operation::Call { signature, .. }) = self.fact(node)?.operation else {
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

impl Builder<'_> {
    pub(super) fn binary_operands(
        &self,
        left: NodeId,
        right: NodeId,
    ) -> Result<crate::body::ir::BinaryOperands, Error> {
        Ok(crate::body::ir::BinaryOperands::new(
            self.immediate_operand(left)?,
            self.immediate_operand(right)?,
        ))
    }
    fn immediate_operand(&self, mut node: NodeId) -> Result<bool, Error> {
        for _ in 0..self.unit.configured.ast.nodes.len() {
            let visits = &self.facts.body_work.cfg_operand_visits;
            visits.set(visits.get().saturating_add(1));
            match self.kind(node) {
                super::Kind::Group(child) | super::Kind::Attributed { value: child, .. } => {
                    node = child;
                }
                super::Kind::Expansion {
                    children,
                    tail: Some(child),
                } if children.start == children.end => node = child,
                super::Kind::Literal(_) | super::Kind::Name(..) | super::Kind::Path { .. } => {
                    return Ok(true);
                }
                super::Kind::Unary {
                    op: crate::syntax::ast::Unary::Negate,
                    ..
                } => {
                    return Ok(matches!(
                        self.fact(node)?.operation,
                        Some(Operation::Unary { literal: true, .. })
                    ));
                }
                _ => return Ok(false),
            }
        }
        Err(self.error(node, "cycle in binary operand source wrappers"))
    }
}
