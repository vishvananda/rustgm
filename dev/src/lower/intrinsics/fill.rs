//! Fill shims use the frozen byte-store vocabulary without typed interpretation.
#[cfg(test)]
#[path = "../../../tests/invariants/fill_shapes.rs"]
mod tests;
use crate::{
    lower::context::{Context, Representation},
    lowir::{
        builder::Builder,
        program::{Function, Operand, Terminator, Value},
        vocabulary::{Binary, Predicate, Type},
    },
};
impl Context<'_> {
    pub(super) fn fill_memory_body(
        representation: Representation,
    ) -> Result<Function, &'static str> {
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let stride = match representation {
            Representation::Scalar(ty) => u64::from(ty.bits() / 8),
            Representation::Object(span) => span.bytes,
            Representation::ScalarObject(storage) => storage.bytes,
        };
        if stride == 0 {
            builder.terminate(Terminator::Return(None))?;
            return builder.finish();
        }
        let count = Value {
            ty: Type::I64,
            operand: Operand::Parameter(2),
        };
        let limit = builder.integer(Type::I64, u128::from(i64::MAX as u64 / stride))?;
        let overflow = builder.compare(Predicate::Ugt, count, limit)?;
        let invalid = builder.block()?;
        let prepare = builder.block()?;
        builder.terminate(Terminator::Branch {
            condition: overflow.operand,
            yes: invalid,
            no: prepare,
        })?;
        builder.enter(invalid)?;
        builder.terminate(Terminator::Unreachable)?;
        builder.enter(prepare)?;
        let stride = builder.integer(Type::I64, u128::from(stride))?;
        let bytes = builder.binary(Binary::Mul, count, stride)?;
        let cursor = builder.slot(Type::I64)?;
        let zero = builder.integer(Type::I64, 0)?;
        builder.store(zero, Operand::Slot(cursor))?;
        let header = builder.block()?;
        let body = builder.block()?;
        let done = builder.block()?;
        builder.terminate(Terminator::Jump(header))?;
        builder.enter(header)?;
        let index = builder.load(Type::I64, Operand::Slot(cursor))?;
        let more = builder.compare(Predicate::Ult, index, bytes)?;
        builder.terminate(Terminator::Branch {
            condition: more.operand,
            yes: body,
            no: done,
        })?;
        builder.enter(body)?;
        let destination = builder.index(
            Type::U8,
            Value {
                ty: Type::Ptr,
                operand: Operand::Parameter(0),
            },
            index,
        )?;
        builder.store(
            Value {
                ty: Type::U8,
                operand: Operand::Parameter(1),
            },
            destination.operand,
        )?;
        let one = builder.integer(Type::I64, 1)?;
        let next = builder.binary(Binary::Add, index, one)?;
        builder.store(next, Operand::Slot(cursor))?;
        builder.terminate(Terminator::Jump(header))?;
        builder.enter(done)?;
        builder.terminate(Terminator::Return(None))?;
        builder.finish()
    }
}
