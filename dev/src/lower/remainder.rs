//! Exact floating remainder by binary significand reduction, including subnormals.
// The helper uses integer operations only; it never rounds x/y before subtraction.
use super::casts;
use crate::lowir::{
    builder::Builder,
    program::{Function, Literal, Operand, Terminator, Value},
    vocabulary::{Binary, Predicate, Type, Unary},
};

fn constant(
    builder: &mut Builder,
    op: Binary,
    value: Value,
    bits: u128,
) -> Result<Value, &'static str> {
    let rhs = builder.integer(value.ty, bits)?;
    builder.binary(op, value, rhs)
}
fn bits(builder: &mut Builder, value: Value) -> Result<Value, &'static str> {
    let ty = if value.ty == Type::F32 {
        Type::U32
    } else {
        Type::I64
    };
    let bits = builder.copy(ty, value)?;
    casts::integer(builder, bits, Type::I64, false)
}
fn returned(builder: &mut Builder, value: Value, ty: Type) -> Result<(), &'static str> {
    let value = if ty == Type::F32 {
        casts::integer(builder, value, Type::U32, false)?
    } else {
        value
    };
    // The pinned backend does not select integer-register to float-register copies.
    // Untyped local storage preserves the bits without asserting C++ alias facts.
    let slot = Operand::Slot(builder.slot(ty)?);
    builder.store(value, slot)?;
    let value = builder.load(ty, slot)?;
    builder.terminate(Terminator::Return(Some(value)))
}
fn normalize(
    builder: &mut Builder,
    significand: Value,
    exponent: Value,
    fraction: u32,
) -> Result<(Value, Value), &'static str> {
    let leading = builder.unary(Unary::Clz, significand)?;
    let shift = constant(builder, Binary::Sub, leading, u128::from(63 - fraction))?;
    let significand = builder.binary(Binary::Shl, significand, shift)?;
    let exponent = builder.binary(Binary::Sub, exponent, shift)?;
    Ok((significand, exponent))
}
fn unpack(
    builder: &mut Builder,
    bits: Value,
    fraction: u32,
) -> Result<(Value, Value), &'static str> {
    let exponent = constant(builder, Binary::Ushr, bits, u128::from(fraction))?;
    let zero = builder.integer(Type::I64, 0)?;
    let normal = builder.compare(Predicate::Ne, exponent, zero)?;
    let subnormal = builder.compare(Predicate::Eq, exponent, zero)?;
    let hidden = constant(builder, Binary::Shl, normal, u128::from(fraction))?;
    let significand = constant(builder, Binary::And, bits, (1_u128 << fraction) - 1)?;
    let significand = builder.binary(Binary::Or, significand, hidden)?;
    let exponent = builder.binary(Binary::Add, exponent, subnormal)?;
    normalize(builder, significand, exponent, fraction)
}
fn subtract(builder: &mut Builder, left: Value, right: Value) -> Result<Value, &'static str> {
    let enough = builder.compare(Predicate::Uge, left, right)?;
    let amount = builder.binary(Binary::Mul, right, enough)?;
    builder.binary(Binary::Sub, left, amount)
}
#[expect(
    clippy::too_many_lines,
    reason = "one cohesive generated CFG keeps shared bit values and block dominance visible"
)]
pub(super) fn build(ty: Type) -> Result<Function, &'static str> {
    let (fraction, infinity, sign_mask, nan_bits) = if ty == Type::F32 {
        (23, 0x7f80_0000, 0x8000_0000, Literal::F32(0x7fc0_0000))
    } else {
        (
            52,
            0x7ff0_0000_0000_0000,
            0x8000_0000_0000_0000,
            Literal::F64(0x7ff8_0000_0000_0000),
        )
    };
    let mut b = Builder::default();
    let entry = b.block()?;
    let exceptional = b.block()?;
    let compare = b.block()?;
    let identity = b.block()?;
    let unpacked = b.block()?;
    let repeat = b.block()?;
    let reduce = b.block()?;
    let tail = b.block()?;
    let zero_result = b.block()?;
    let encode = b.block()?;
    let normal = b.block()?;
    let subnormal = b.block()?;
    let sig_slot = Operand::Slot(b.slot(Type::I64)?);
    let exp_slot = Operand::Slot(b.slot(Type::I64)?);
    b.enter(entry)?;
    let x = Value {
        ty,
        operand: Operand::Parameter(0),
    };
    let y = Value {
        ty,
        operand: Operand::Parameter(1),
    };
    let raw_x = bits(&mut b, x)?;
    let raw_y = bits(&mut b, y)?;
    let sign = constant(&mut b, Binary::And, raw_x, sign_mask)?;
    let abs_x = constant(&mut b, Binary::And, raw_x, sign_mask - 1)?;
    let abs_y = constant(&mut b, Binary::And, raw_y, sign_mask - 1)?;
    let inf = b.integer(Type::I64, infinity)?;
    let zero = b.integer(Type::I64, 0)?;
    let x_invalid = b.compare(Predicate::Uge, abs_x, inf)?;
    let y_nan = b.compare(Predicate::Ugt, abs_y, inf)?;
    let y_zero = b.compare(Predicate::Eq, abs_y, zero)?;
    let fault = b.binary(Binary::Or, x_invalid, y_nan)?;
    let fault = b.binary(Binary::Or, fault, y_zero)?;
    b.terminate(Terminator::Branch {
        condition: fault.operand,
        yes: exceptional,
        no: compare,
    })?;
    b.enter(exceptional)?;
    let nan = b.literal(ty, nan_bits)?;
    b.terminate(Terminator::Return(Some(nan)))?;
    b.enter(compare)?;
    let smaller = b.compare(Predicate::Ult, abs_x, abs_y)?;
    b.terminate(Terminator::Branch {
        condition: smaller.operand,
        yes: identity,
        no: unpacked,
    })?;
    b.enter(identity)?;
    b.terminate(Terminator::Return(Some(x)))?;
    b.enter(unpacked)?;
    let (sx, ex) = unpack(&mut b, abs_x, fraction)?;
    let (sy, ey) = unpack(&mut b, abs_y, fraction)?;
    b.store(sx, sig_slot)?;
    b.store(ex, exp_slot)?;
    b.terminate(Terminator::Jump(repeat))?;
    b.enter(repeat)?;
    let sx = b.load(Type::I64, sig_slot)?;
    let ex = b.load(Type::I64, exp_slot)?;
    let again = b.compare(Predicate::Gt, ex, ey)?;
    b.terminate(Terminator::Branch {
        condition: again.operand,
        yes: reduce,
        no: tail,
    })?;
    b.enter(reduce)?;
    let remainder = subtract(&mut b, sx, sy)?;
    let shifted = constant(&mut b, Binary::Shl, remainder, 1)?;
    let decremented = constant(&mut b, Binary::Sub, ex, 1)?;
    b.store(shifted, sig_slot)?;
    b.store(decremented, exp_slot)?;
    b.terminate(Terminator::Jump(repeat))?;
    b.enter(tail)?;
    let remainder = subtract(&mut b, sx, sy)?;
    let is_zero = b.compare(Predicate::Eq, remainder, zero)?;
    b.terminate(Terminator::Branch {
        condition: is_zero.operand,
        yes: zero_result,
        no: encode,
    })?;
    b.enter(zero_result)?;
    returned(&mut b, sign, ty)?;
    b.enter(encode)?;
    let (significand, exponent) = normalize(&mut b, remainder, ex, fraction)?;
    let is_normal = b.compare(Predicate::Gt, exponent, zero)?;
    b.terminate(Terminator::Branch {
        condition: is_normal.operand,
        yes: normal,
        no: subnormal,
    })?;
    b.enter(normal)?;
    let fraction_bits = constant(&mut b, Binary::And, significand, (1_u128 << fraction) - 1)?;
    let exponent_bits = constant(&mut b, Binary::Shl, exponent, u128::from(fraction))?;
    let result = b.binary(Binary::Or, fraction_bits, exponent_bits)?;
    let result = b.binary(Binary::Or, result, sign)?;
    returned(&mut b, result, ty)?;
    b.enter(subnormal)?;
    let one = b.integer(Type::I64, 1)?;
    let shift = b.binary(Binary::Sub, one, exponent)?;
    // A nonzero remainder is a multiple of the minimum subnormal, so shift <= fraction.
    let result = b.binary(Binary::Ushr, significand, shift)?;
    let result = b.binary(Binary::Or, result, sign)?;
    returned(&mut b, result, ty)?;
    b.finish()
}
