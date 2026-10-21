//! Bitwise RMW retries CAS using the backend-updated local expected value.
use crate::{
    lowir::{
        atomic::{CompareOrder, LoadOrder, RmwOrder},
        builder::Builder,
        program::{Operand, SlotId, Terminator, Value},
        vocabulary::{Binary, Type},
    },
    support::intrinsics::Intrinsic,
};
pub(super) fn operation(
    builder: &mut Builder,
    intrinsic: Intrinsic,
    value: Value,
    pointer: Operand,
    order: RmwOrder,
    expected: SlotId,
) -> Result<Value, &'static str> {
    let seed = builder.atomic_load(Type::U32, pointer, LoadOrder::Relaxed)?;
    builder.store(seed, Operand::Slot(expected))?;
    let address = builder.address(Operand::Slot(expected))?;
    let retry = builder.block()?;
    let done = builder.block()?;
    builder.terminate(Terminator::Jump(retry))?;
    builder.enter(retry)?;
    let previous = builder.load(Type::U32, Operand::Slot(expected))?;
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
    let flag = builder.atomic_compare_exchange(
        updated,
        pointer,
        address.operand,
        CompareOrder::new(order, LoadOrder::Relaxed),
    )?;
    // Failure updates only local scratch. The next iteration reloads it;
    // source arguments are never evaluated again and success returns old bits.
    builder.terminate(Terminator::Branch {
        condition: flag.operand,
        yes: done,
        no: retry,
    })?;
    builder.enter(done)?;
    Ok(previous)
}
