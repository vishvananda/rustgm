//! Unsigned extrema use a wrapping U32 mask without another atomic-object access.
use crate::{
    lowir::{
        builder::Builder,
        program::Value,
        vocabulary::{Binary, Convert, Predicate, Type},
    },
    support::intrinsics::Intrinsic,
};
pub(super) fn update(
    builder: &mut Builder,
    intrinsic: Intrinsic,
    previous: Value,
    value: Value,
) -> Result<Value, &'static str> {
    if previous.ty != Type::U32 || value.ty != Type::U32 {
        return Err("atomic extremum requires u32 operands");
    }
    let predicate = match intrinsic {
        Intrinsic::AtomicFetchMin => Predicate::Ugt,
        Intrinsic::AtomicFetchMax => Predicate::Ult,
        _ => return Err("invalid atomic extremum identity"),
    };
    let replace = builder.compare(predicate, previous, value)?;
    // LowIR comparisons produce canonical I64 0/1. Truncation preserves that
    // boolean while the subtraction below deliberately wraps at U32 width.
    let replace = builder.convert(Convert::Trunc, Type::U32, replace)?;
    let zero = builder.integer(Type::U32, 0)?;
    let mask = builder.binary(Binary::Sub, zero, replace)?;
    let difference = builder.binary(Binary::Xor, previous, value)?;
    let selected = builder.binary(Binary::And, difference, mask)?;
    builder.binary(Binary::Xor, previous, selected)
}
