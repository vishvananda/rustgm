//! Overlap-safe moves use ordinary byte transfers in the selected direction.
#[cfg(test)]
#[path = "../../../tests/invariants/overlapping_shapes.rs"]
mod tests;
use crate::{
    lower::context::{Context, Representation},
    lowir::{
        builder::Builder,
        program::{BlockId, Function, Operand, SlotId, Terminator, Value},
        vocabulary::{Binary, Predicate, Type},
    },
};
impl Context<'_> {
    pub(super) fn overlapping_memory_body(
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
        let stride = builder.integer(Type::I64, u128::from(stride))?;
        let bytes = builder.binary(Binary::Mul, count, stride)?;
        let zero = builder.integer(Type::I64, 0)?;
        let empty = builder.compare(Predicate::Eq, bytes, zero)?;
        let done = builder.block()?;
        let choose = builder.block()?;
        builder.terminate(Terminator::Branch {
            condition: empty.operand,
            yes: done,
            no: choose,
        })?;
        builder.enter(choose)?;
        let backward = builder.compare(Predicate::Ugt, pointer(1), pointer(0))?;
        let front = builder.block()?;
        let back = builder.block()?;
        builder.terminate(Terminator::Branch {
            condition: backward.operand,
            yes: back,
            no: front,
        })?;
        let cursor = builder.slot(Type::I64)?;
        byte_loop::<false>(&mut builder, front, done, cursor, bytes)?;
        byte_loop::<true>(&mut builder, back, done, cursor, bytes)?;
        builder.enter(done)?;
        builder.terminate(Terminator::Return(None))?;
        builder.finish()
    }
}
fn pointer(parameter: u32) -> Value {
    Value {
        ty: Type::Ptr,
        operand: Operand::Parameter(parameter),
    }
}
fn byte_loop<const BACKWARD: bool>(
    builder: &mut Builder,
    start: BlockId,
    done: BlockId,
    cursor: SlotId,
    bytes: Value,
) -> Result<(), &'static str> {
    builder.enter(start)?;
    let zero = builder.integer(Type::I64, 0)?;
    builder.store(if BACKWARD { bytes } else { zero }, Operand::Slot(cursor))?;
    let header = builder.block()?;
    let body = builder.block()?;
    builder.terminate(Terminator::Jump(header))?;
    builder.enter(header)?;
    let offset = builder.load(Type::I64, Operand::Slot(cursor))?;
    let more = if BACKWARD {
        builder.compare(Predicate::Ne, offset, zero)?
    } else {
        builder.compare(Predicate::Ult, offset, bytes)?
    };
    builder.terminate(Terminator::Branch {
        condition: more.operand,
        yes: body,
        no: done,
    })?;
    builder.enter(body)?;
    let one = builder.integer(Type::I64, 1)?;
    let index = if BACKWARD {
        builder.binary(Binary::Sub, offset, one)?
    } else {
        offset
    };
    let source = builder.index(Type::U8, pointer(0), index)?;
    let destination = builder.index(Type::U8, pointer(1), index)?;
    let value = builder.load(Type::U8, source.operand)?;
    builder.store(value, destination.operand)?;
    let next = if BACKWARD {
        index
    } else {
        builder.binary(Binary::Add, offset, one)?
    };
    builder.store(next, Operand::Slot(cursor))?;
    builder.terminate(Terminator::Jump(header))
}
