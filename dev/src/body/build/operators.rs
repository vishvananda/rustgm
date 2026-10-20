//! Unary syntax consumes its selected scalar, reference or raw-address operation.
use super::{Builder, Error, NodeId, Operation, Rvalue, ValueId};
use crate::sema::model::Cast;

impl Builder<'_> {
    pub(super) fn unary_value(&mut self, node: NodeId, operand: NodeId) -> Result<ValueId, Error> {
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
