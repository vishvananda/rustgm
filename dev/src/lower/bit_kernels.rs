//! Parallel bit-group transforms use logarithmic fixed-width scalar work.
use crate::{
    lowir::{
        builder::Builder,
        program::{Function, Operand, Terminator, Value},
        vocabulary::{Binary, Type},
    },
    support::llvm_symbols::Operation,
};
pub(super) fn build(
    operation: Operation,
    ty: Type,
    result: Type,
    bits: u16,
) -> Result<Function, &'static str> {
    if operation == Operation::Unsupported
        || !matches!(bits, 8 | 16 | 32 | 64 | 128)
        || u32::from(bits) != ty.bits()
        || result.bits() != ty.bits()
    {
        return Err("invalid scalar LLVM kernel descriptor");
    }
    let mut builder = Builder::default();
    let block = builder.block()?;
    builder.enter(block)?;
    let input = Value {
        ty,
        operand: Operand::Parameter(0),
    };
    let mut value = emit(&mut builder, operation, input)?;
    if value.ty != result {
        value = builder.copy(result, value)?;
    }
    builder.terminate(Terminator::Return(Some(value)))?;
    builder.finish()
}
/// The transform of an integer value at its own width, in the open block.
pub(super) fn emit(
    builder: &mut Builder,
    operation: Operation,
    mut value: Value,
) -> Result<Value, &'static str> {
    let ty = value.ty;
    let bits = match ty.bits() {
        8 => 8_u16,
        16 => 16,
        32 => 32,
        64 => 64,
        128 => 128,
        _ => return Err("invalid scalar LLVM kernel descriptor"),
    };
    if operation == Operation::Unsupported {
        return Err("invalid scalar LLVM kernel descriptor");
    }
    let mut shift = if operation == Operation::SwapBytes {
        8
    } else {
        1
    };
    while shift < bits {
        let mut mask = 0_u128;
        for offset in (0..bits).step_by(usize::from(2 * shift)) {
            mask |= ((1_u128 << shift) - 1) << offset;
        }
        let mask = builder.integer(ty, mask)?;
        let count = builder.integer(ty, u128::from(shift))?;
        let high = builder.binary(Binary::Ushr, value, count)?;
        let high = builder.binary(Binary::And, high, mask)?;
        let low = builder.binary(Binary::And, value, mask)?;
        value = if operation == Operation::Population {
            builder.binary(Binary::Add, low, high)?
        } else {
            let low = builder.binary(Binary::Shl, low, count)?;
            builder.binary(Binary::Or, low, high)?
        };
        shift *= 2;
    }
    Ok(value)
}
