//! Reconstructed mutable references retain the existing pointer's authority.
use super::{Failure, Machine, RawAccess, Resolved, TypeId};
use crate::{
    body, sema,
    sema::values::{Constant, WritePermission},
    test_source,
    types::Primitive,
};
#[test]
fn mutable_reference_preserves_authority_and_rejects_shared_input() -> Result<(), String> {
    let report = test_source::resolve("const fn f(){}", true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let bodies = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let ty = TypeId::primitive(Primitive::U32);
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, &bodies.bodies, super::super::Limits::default())?;
    let mut allocations = Vec::new();
    let value = Constant::scalar(sema::model::Scalar::Integer(37));
    let address = machine
        .allocate(&mut allocations, Some(value), ty)
        .map_err(Failure::message)?;
    let place = Resolved {
        address,
        ty,
        union: None,
        raw: Some(RawAccess {
            location: address.location(),
            ty,
        }),
    };
    let result = machine
        .mutable_raw_reference(place)
        .map_err(Failure::message)?
        .ok_or("mutable reference")?;
    assert!(result == address);
    assert_eq!(result.write_permission(), Some(WritePermission::Writable));
    let shared = machine
        .shared_raw_reference(place)
        .map_err(Failure::message)?
        .ok_or("shared reference")?;
    assert_eq!(shared.location(), address.location());
    assert_eq!(
        shared.write_permission(),
        Some(WritePermission::SharedDerived)
    );
    assert_eq!(
        machine
            .mutable_raw_reference(Resolved {
                address: shared,
                ..place
            })
            .err()
            .map(Failure::message),
        Some("constant mutable raw references from shared-derived pointers are not implemented")
    );
    assert!(
        machine
            .mutable_raw_reference(place)
            .map_err(Failure::message)?
            .ok_or("original alias")?
            == address
    );
    machine.memory.expire(&allocations);
    assert_eq!(
        machine
            .mutable_raw_reference(place)
            .err()
            .map(Failure::message),
        Some("constant pointer refers to expired storage")
    );
    Ok(())
}
