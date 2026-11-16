//! One SSE operation on `v128` values, typed by its shape before it is emitted.
use super::{
    super::vocabulary::{Vector, VectorShape},
    Builder, Operation, Type, Value,
};
impl Builder {
    /// Emit `vector op`: `right` is the second operand of a binary shape and
    /// `immediate` the byte of an immediate shape.
    ///
    /// # Errors
    /// Rejects operands whose types or presence differ from the shape and
    /// invalid construction state.
    pub fn vector(
        &mut self,
        op: Vector,
        left: Value,
        right: Option<Value>,
        immediate: u8,
    ) -> Result<Value, &'static str> {
        let binary = op.shape() == VectorShape::Binary;
        let immediate_shape = op.shape() == VectorShape::UnaryImmediate;
        if left.ty != op.operand()
            || right.is_some() != binary
            || right.is_some_and(|right| right.ty != Type::V128)
            || (!immediate_shape && immediate != 0)
        {
            return Err("LowIR vector operands differ from their operation's shape");
        }
        self.value(
            op.result(),
            Operation::Vector {
                op,
                left: left.operand,
                right: right.map(|right| right.operand),
                immediate,
            },
        )
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/vector_values.rs"]
mod tests;
