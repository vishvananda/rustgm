//! Pure bitwise updates share the common atomic CAS retry scaffold.
use crate::{
    lowir::{
        builder::Builder,
        program::Value,
        vocabulary::{Binary, Type},
    },
    support::intrinsics::Intrinsic,
};
pub(super) fn update(
    builder: &mut Builder,
    intrinsic: Intrinsic,
    previous: Value,
    value: Value,
) -> Result<Value, &'static str> {
    let op = match intrinsic {
        Intrinsic::AtomicFetchAnd | Intrinsic::AtomicFetchNand => Binary::And,
        Intrinsic::AtomicFetchOr => Binary::Or,
        Intrinsic::AtomicFetchXor => Binary::Xor,
        _ => return Err("invalid bitwise atomic identity"),
    };
    let mut updated = builder.binary(op, previous, value)?;
    if intrinsic == Intrinsic::AtomicFetchNand {
        let mask = builder.integer(Type::U32, u128::from(u32::MAX))?;
        updated = builder.binary(Binary::Xor, updated, mask)?;
    }
    Ok(updated)
}
