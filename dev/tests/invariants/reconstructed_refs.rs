//! A valid packed-base by-value read does not establish leaf reference alignment.
use super::{Failure, Machine, RawAccess, Resolved, TypeId};
use crate::{
    body, sema,
    sema::values::Constant,
    test_source,
    types::{Kind, Primitive},
};
#[test]
fn packed_raw_base_read_and_leaf_reference_require_distinct_alignment() -> Result<(), String> {
    let report = test_source::resolve(
        "#[repr(C,packed)]struct P{prefix:u8,x:u32}const fn f(){let x=P{prefix:0,x:37};let _=&x;}",
        true,
    )?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let bodies = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let packed = bodies
        .bodies
        .iter()
        .flat_map(|b| &b.values)
        .find_map(|v| {
            matches!(v.kind, body::ir::Rvalue::Aggregate { .. })
                .then_some(v.ty)
                .filter(|&ty| matches!(facts.types.kind(ty), Some(Kind::Adt(..))))
        })
        .ok_or("packed type")?;
    let ty = TypeId::primitive(Primitive::U32);
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, &bodies.bodies, super::super::Limits::default())?;
    let value = machine
        .evaluator
        .fields(
            &[
                Constant::scalar(sema::model::Scalar::Integer(0)),
                Constant::scalar(sema::model::Scalar::Integer(37)),
            ],
            None,
        )
        .map_err(Failure::message)?;
    let mut allocations = Vec::new();
    let address = machine
        .allocate(&mut allocations, Some(value), packed)
        .map_err(Failure::message)?;
    let field = machine
        .evaluator
        .projection(address, 1, ty)
        .map_err(Failure::message)?;
    let place = Resolved {
        address: field,
        ty,
        union: None,
        raw: Some(RawAccess {
            location: address.location(),
            ty: packed,
        }),
    };
    assert_eq!(
        machine
            .read(place)
            .map_err(Failure::message)?
            .ok_or("read")?
            .as_scalar()
            .map_err(Failure::message)?,
        sema::model::Scalar::Integer(37)
    );
    assert_eq!(
        machine
            .shared_raw_reference(place)
            .err()
            .map(Failure::message),
        Some("constant raw shared reference is unaligned")
    );
    assert_eq!(
        machine
            .shared_raw_reference(Resolved { raw: None, ..place })
            .err()
            .map(Failure::message),
        Some("constant raw shared reference lacks its access anchor")
    );
    Ok(())
}
