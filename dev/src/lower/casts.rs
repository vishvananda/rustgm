//! Selected scalar casts, including Rust's saturating floating conversions.
use crate::{
    lowir::{
        builder::Builder,
        program::{Literal, Operand, Terminator, Value},
        vocabulary::{Convert, Predicate, Type},
    },
    sema::model::Cast,
    types::Primitive,
};

pub(super) fn integer(
    builder: &mut Builder,
    value: Value,
    target: Type,
    signed: bool,
) -> Result<Value, &'static str> {
    if value.ty == target {
        return Ok(value);
    }
    if value.ty.bits() == target.bits() {
        return builder.copy(target, value);
    }
    let op = if value.ty.bits() > target.bits() {
        Convert::Trunc
    } else if signed {
        Convert::Sext
    } else {
        Convert::Zext
    };
    builder.convert(op, target, value)
}
pub(super) fn cast(
    builder: &mut Builder,
    kind: Cast,
    source: Primitive,
    target: Primitive,
    value: Value,
    ty: Type,
) -> Result<Value, &'static str> {
    let signed = source.integer().is_some_and(|(signed, _)| signed);
    match kind {
        Cast::IntegerToInteger | Cast::BoolToInteger | Cast::CharToInteger | Cast::U8ToChar => {
            integer(builder, value, ty, signed)
        }
        Cast::IntegerToFloat => builder.convert(
            if signed {
                Convert::Sitofp
            } else {
                Convert::Uitofp
            },
            ty,
            value,
        ),
        Cast::FloatToFloat => {
            if value.ty == ty {
                Ok(value)
            } else {
                builder.convert(
                    if ty == Type::F64 {
                        Convert::Fpext
                    } else {
                        Convert::Fptrunc
                    },
                    ty,
                    value,
                )
            }
        }
        Cast::FloatToInteger => saturate(builder, value, ty, target),
        Cast::FunctionPointer => Err("function reification is not a numeric cast"),
    }
}
fn float(
    builder: &mut Builder,
    ty: Type,
    exponent: u32,
    negative: bool,
) -> Result<Value, &'static str> {
    let sign = u64::from(negative) << 63;
    let bits = sign | (u64::from(exponent + 1023) << 52);
    if ty == Type::F64 {
        builder.literal(ty, Literal::F64(bits))
    } else {
        // Powers of two are exact, or infinity at f32's upper saturation boundary.
        let raw = (u32::from(negative) << 31) | ((exponent + 127).min(255) << 23);
        builder.literal(ty, Literal::F32(raw))
    }
}
fn saturate(
    builder: &mut Builder,
    value: Value,
    target: Type,
    primitive: Primitive,
) -> Result<Value, &'static str> {
    let (signed, bits) = primitive
        .integer()
        .ok_or("invalid floating conversion target")?;
    let width = u32::from(bits);
    let maximum = if signed {
        (1_u128 << (width - 1)) - 1
    } else {
        u128::MAX >> (128 - width)
    };
    let minimum = if signed { 1_u128 << (width - 1) } else { 0 };
    let slot = Operand::Slot(builder.slot(target)?);
    let nan = builder.block()?;
    let high_check = builder.block()?;
    let high = builder.block()?;
    let low_check = builder.block()?;
    let low = builder.block()?;
    let convert = builder.block()?;
    let done = builder.block()?;
    let is_nan = builder.compare(Predicate::Ne, value, value)?;
    builder.terminate(Terminator::Branch {
        condition: is_nan.operand,
        yes: nan,
        no: high_check,
    })?;
    builder.enter(nan)?;
    let zero = builder.integer(target, 0)?;
    builder.store(zero, slot)?;
    builder.terminate(Terminator::Jump(done))?;
    builder.enter(high_check)?;
    let upper = float(builder, value.ty, width - u32::from(signed), false)?;
    let at_high = builder.compare(Predicate::Ge, value, upper)?;
    builder.terminate(Terminator::Branch {
        condition: at_high.operand,
        yes: high,
        no: low_check,
    })?;
    builder.enter(high)?;
    let max = builder.integer(target, maximum)?;
    builder.store(max, slot)?;
    builder.terminate(Terminator::Jump(done))?;
    builder.enter(low_check)?;
    let lower = if signed {
        float(builder, value.ty, width - 1, true)?
    } else {
        builder.literal(
            value.ty,
            if value.ty == Type::F64 {
                Literal::F64(0)
            } else {
                Literal::F32(0)
            },
        )?
    };
    let at_low = builder.compare(Predicate::Le, value, lower)?;
    builder.terminate(Terminator::Branch {
        condition: at_low.operand,
        yes: low,
        no: convert,
    })?;
    builder.enter(low)?;
    let min = builder.integer(target, minimum)?;
    builder.store(min, slot)?;
    builder.terminate(Terminator::Jump(done))?;
    builder.enter(convert)?;
    let result = builder.convert(
        if signed {
            Convert::Fptosi
        } else {
            Convert::Fptoui
        },
        target,
        value,
    )?;
    builder.store(result, slot)?;
    builder.terminate(Terminator::Jump(done))?;
    builder.enter(done)?;
    builder.load(target, slot)
}
