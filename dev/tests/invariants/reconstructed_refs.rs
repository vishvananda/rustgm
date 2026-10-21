//! A valid packed-base by-value read does not establish leaf reference alignment.
use super::{Failure, Machine, RawAccess, Resolved, TypeId};
use crate::{
    body, sema,
    sema::values::Constant,
    test_source,
    types::{Kind, Primitive, RegionId, Store},
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

#[test]
fn reference_cell_storage_equivalence_ignores_only_its_outer_region() -> Result<(), String> {
    let report = test_source::resolve("const fn f(){}", true)?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let bodies = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let scalar = TypeId::primitive(Primitive::U32);
    let signed = TypeId::primitive(Primitive::I32);
    let a = reference(&mut facts.types, 1, false, scalar)?;
    let b = reference(&mut facts.types, 2, false, scalar)?;
    let mutable = reference(&mut facts.types, 2, true, scalar)?;
    let other = reference(&mut facts.types, 2, false, signed)?;
    let nested_a = reference(&mut facts.types, 1, false, a)?;
    let nested_b = reference(&mut facts.types, 2, false, b)?;
    let pointer_a = facts.types.intern(Kind::Pointer {
        mutable: false,
        pointee: a,
    })?;
    let pointer_b = facts.types.intern(Kind::Pointer {
        mutable: false,
        pointee: b,
    })?;
    assert_ne!(a, b);
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let machine = Machine::new(evaluator, &bodies.bodies, super::super::Limits::default())?;
    assert!(machine.same_raw_storage_type(a, a));
    assert!(machine.same_raw_storage_type(a, b));
    assert!(machine.same_raw_storage_type(b, a));
    for (left, right) in [
        (a, mutable),
        (mutable, a),
        (a, other),
        (nested_a, nested_b),
        (pointer_a, pointer_b),
        (scalar, signed),
    ] {
        assert!(!machine.same_raw_storage_type(left, right));
    }
    Ok(())
}

fn reference(
    types: &mut Store,
    region: u32,
    mutable: bool,
    referent: TypeId,
) -> Result<TypeId, String> {
    types
        .intern(Kind::Reference {
            region: RegionId(region),
            mutable,
            referent,
        })
        .map_err(str::to_owned)
}
