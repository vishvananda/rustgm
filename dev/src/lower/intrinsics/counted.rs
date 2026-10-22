//! Counted non-overlapping transfers use the frozen scalar/object vocabulary.
#[cfg(test)]
#[path = "../../../tests/invariants/counted_shapes.rs"]
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
    pub(super) fn counted_memory_body(
        representation: Representation,
    ) -> Result<Function, &'static str> {
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let bytes = match representation {
            Representation::Scalar(ty) => u64::from(ty.bits() / 8),
            Representation::Object(span) => span.bytes,
        };
        if bytes == 0 {
            builder.terminate(Terminator::Return(None))?;
            return builder.finish();
        }
        let cursor = builder.slot(Type::I64)?;
        let zero = builder.integer(Type::I64, 0)?;
        builder.store(zero, Operand::Slot(cursor))?;
        let header = builder.block()?;
        let body = builder.block()?;
        let done = builder.block()?;
        builder.terminate(Terminator::Jump(header))?;
        builder.enter(header)?;
        let index = builder.load(Type::I64, Operand::Slot(cursor))?;
        let count = Value {
            ty: Type::I64,
            operand: Operand::Parameter(2),
        };
        let active = builder.compare(Predicate::Ult, index, count)?;
        builder.terminate(Terminator::Branch {
            condition: active.operand,
            yes: body,
            no: done,
        })?;
        builder.enter(body)?;
        let stride = builder.integer(Type::I64, u128::from(bytes))?;
        let offset = builder.binary(Binary::Mul, index, stride)?;
        let source = builder.index(
            Type::U8,
            Value {
                ty: Type::Ptr,
                operand: Operand::Parameter(0),
            },
            offset,
        )?;
        let destination = builder.index(
            Type::U8,
            Value {
                ty: Type::Ptr,
                operand: Operand::Parameter(1),
            },
            offset,
        )?;
        match representation {
            Representation::Scalar(ty) => {
                let value = builder.load(ty, source.operand)?;
                builder.store(value, destination.operand)?;
            }
            Representation::Object(span) => builder.copy_object(span, source, destination)?,
        }
        let one = builder.integer(Type::I64, 1)?;
        let next = builder.binary(Binary::Add, index, one)?;
        builder.store(next, Operand::Slot(cursor))?;
        builder.terminate(Terminator::Jump(header))?;
        builder.enter(done)?;
        builder.terminate(Terminator::Return(None))?;
        builder.finish()
    }
}
