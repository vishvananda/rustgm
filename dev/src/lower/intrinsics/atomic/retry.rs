//! RMW retries consume backend-updated expected scratch and return captured old bits.
use crate::{
    lowir::{
        atomic::{CompareOrder, LoadOrder, RmwOrder},
        builder::Builder,
        program::{Operand, SlotId, Terminator, Value},
        vocabulary::Type,
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
    let updated = if intrinsic.is_bitwise() {
        super::bitwise::update(builder, intrinsic, previous, value)?
    } else {
        super::extremum::update(builder, intrinsic, previous, value)?
    };
    let flag = builder.atomic_compare_exchange(
        updated,
        pointer,
        address.operand,
        CompareOrder::new(order, LoadOrder::Relaxed),
    )?;
    // Failed CAS updates expected; only that scratch is reloaded. The source
    // operand is captured once and even unchanged desired bits perform CAS.
    builder.terminate(Terminator::Branch {
        condition: flag.operand,
        yes: done,
        no: retry,
    })?;
    builder.enter(done)?;
    Ok(previous)
}
