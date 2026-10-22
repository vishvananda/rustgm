//! No-access fills retain alignment but never create ordinary write authority.
use super::{Constant, Failure, Machine};
use crate::{
    const_eval::Limits,
    sema::{self, model::Scalar, values::Evaluator},
    test_source,
    types::{Primitive, TypeId},
};
fn integer(value: u128) -> Constant {
    Constant::scalar(Scalar::Integer(value))
}
#[test]
fn fill_empty_access_preserves_expiry_and_each_alias_permission() -> Result<(), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
    let ty = TypeId::primitive(Primitive::U32);
    let mut allocations = Vec::new();
    let pointer = machine
        .allocate(&mut allocations, Some(integer(7)), ty)
        .map_err(Failure::message)?;
    let shared = pointer.restrict_shared();
    assert!(
        machine
            .counted_fill(ty, &[shared, integer(2), integer(0)])
            .map_err(Failure::message)?
            .is_some()
    );
    assert!(!machine.has_byte_root(pointer.location().ok_or("root")?));
    assert_eq!(
        machine
            .counted_fill(ty, &[shared, integer(2), integer(1)])
            .err()
            .map(Failure::message),
        Some("constant raw pointer write is derived from a shared reference")
    );
    machine
        .counted_fill(ty, &[pointer, integer(2), integer(1)])
        .map_err(Failure::message)?;
    assert_eq!(machine.slots, 0);
    let before = machine.memory.retained_cells();
    machine.limits.slots = before;
    // A retained span cannot fit. Failure must neither retain it nor modify bytes.
    assert!(
        machine
            .counted_fill(ty, &[pointer, integer(3), integer(1)])
            .is_err()
    );
    assert_eq!(machine.slots, 0);
    machine.limits.slots = Limits::default().slots;
    let snapshot = machine
        .snapshot_byte_root(pointer.location().ok_or("root")?, 0, 4)
        .map_err(Failure::message)?
        .ok_or("snapshot")?;
    assert_eq!(snapshot.bytes, [2; 4]);
    machine.memory.expire(&allocations);
    assert!(
        machine
            .counted_fill(ty, &[shared, integer(2), integer(0)])
            .map_err(Failure::message)?
            .is_some()
    );
    assert_eq!(
        machine
            .counted_fill(ty, &[pointer, integer(2), integer(1)])
            .err()
            .map(Failure::message),
        Some("constant pointer refers to expired storage")
    );
    let span = machine
        .transfer_span(pointer, ty, 0, 0, 0)
        .map_err(Failure::message)?;
    assert!(machine.read(span.place()).is_err());
    assert!(machine.check_raw_write(span.place()).is_err());
    assert_eq!(machine.slots, 0);
    Ok(())
}
