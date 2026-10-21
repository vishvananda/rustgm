//! Checked symbolic cells independently enforce liveness, initialization and path bounds.
use super::{Failure, Machine, Resolved, TypeId};
use crate::{body, sema, sema::values::Constant, test_source, types::Primitive};
#[test]
fn initialized_storage_is_required_before_a_raw_read() -> Result<(), String> {
    let report = test_source::resolve("const fn f(){}", true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let bodies = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let ty = TypeId::primitive(Primitive::U32);
    let array_ty = facts.types.intern(crate::types::Kind::Array {
        element: ty,
        length: 1,
    })?;
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, &bodies.bodies, super::super::Limits::default())?;
    let mut allocations = Vec::new();
    let address = machine
        .allocate(&mut allocations, None, ty)
        .map_err(Failure::message)?;
    let place = Resolved {
        address,
        ty,
        raw: true,
    };
    assert_eq!(
        machine.read(place).err().map(Failure::message),
        Some("constant reads uninitialized storage")
    );
    machine
        .write(address, Constant::scalar(sema::model::Scalar::Integer(37)))
        .map_err(Failure::message)?;
    assert_eq!(
        machine
            .read(place)
            .map_err(Failure::message)?
            .ok_or("missing read")?
            .as_scalar()
            .map_err(Failure::message)?,
        sema::model::Scalar::Integer(37)
    );
    let layout_steps = machine.evaluator.facts().layouts.steps;
    for _ in 0..8 {
        machine
            .read(place)
            .map_err(Failure::message)?
            .ok_or("missing cached read")?;
    }
    assert_eq!(machine.evaluator.facts().layouts.steps, layout_steps);
    let value = machine
        .evaluator
        .repeat(Constant::scalar(sema::model::Scalar::Integer(37)), 1)
        .map_err(Failure::message)?;
    let array = machine
        .allocate(&mut allocations, Some(value), array_ty)
        .map_err(Failure::message)?;
    let invalid = machine
        .evaluator
        .projection(array, 1, ty)
        .map_err(Failure::message)?;
    assert_eq!(
        machine
            .read(Resolved {
                address: invalid,
                ty,
                raw: true
            })
            .err()
            .map(Failure::message),
        Some("constant index is out of bounds")
    );
    machine.memory.expire(&allocations);
    assert_eq!(
        machine.read(place).err().map(Failure::message),
        Some("constant pointer refers to expired storage")
    );
    Ok(())
}
