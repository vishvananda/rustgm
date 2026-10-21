//! Static root-property caching charges retained facts without caching access authority.
use super::{Failure, Machine, ReadPurpose};
use crate::{
    body, sema, test_source,
    types::{Kind, Primitive, TypeId},
};

#[test]
fn repeated_static_checks_walk_each_root_type_once() -> Result<(), String> {
    let report = test_source::resolve(
        "use core::sync::atomic::AtomicU32;const fn f(_: &AtomicU32){}",
        true,
    )?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let bodies = body::build(&report, &facts).map_err(|e| e.to_string())?;
    assert!(facts.aggregates.has_shared_mutation());
    let scalar = TypeId::primitive(Primitive::U32);
    let mut roots = Vec::new();
    for width in [2, 16, 64] {
        let fields = facts.types.intern_list(&vec![scalar; width])?;
        roots.push(facts.types.intern(Kind::Tuple(fields))?);
    }
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, &bodies.bodies, super::super::Limits::default())?;
    let mut repeated = None;
    for (index, root) in roots.into_iter().enumerate() {
        let before = machine.work().steps;
        machine
            .check_static_value_read(root, scalar, ReadPurpose::Value)
            .map_err(Failure::message)?;
        let first = machine.work().steps - before;
        let before = machine.work().steps;
        for _ in 0..32 {
            machine
                .check_static_value_read(root, scalar, ReadPurpose::Value)
                .map_err(Failure::message)?;
        }
        let steady = machine.work().steps - before;
        assert!(first > steady / 32);
        if let Some(previous) = repeated {
            assert_eq!(steady, previous);
        }
        repeated = Some(steady);
        assert_eq!(machine.memory.ordinary_static_types.len(), index + 1);
        println!("root={index} first={first} repeated32={steady}");
    }
    Ok(())
}
#[test]
fn exhausted_static_fact_storage_does_not_publish_or_poison_other_roots() -> Result<(), String> {
    let report = test_source::resolve(
        "use core::sync::atomic::AtomicU32;const fn f(_: &AtomicU32){}",
        true,
    )?;
    let facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let mut facts = sema::bodies::analyze(&report, facts).map_err(|e| e.to_string())?;
    let bodies = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(
        evaluator,
        &bodies.bodies,
        super::super::Limits {
            slots: 0,
            ..Default::default()
        },
    )?;
    let first = TypeId::primitive(Primitive::U32);
    let second = TypeId::primitive(Primitive::U64);
    assert_eq!(
        machine
            .check_static_value_read(first, first, ReadPurpose::Value)
            .err()
            .map(Failure::message),
        Some("constant execution storage limit exceeded")
    );
    assert!(machine.memory.ordinary_static_types.is_empty());
    machine.limits.slots = 4;
    machine
        .check_static_value_read(second, second, ReadPurpose::Value)
        .map_err(Failure::message)?;
    assert_eq!(machine.memory.retained_cells(), 4);
    assert_eq!(
        machine
            .check_static_value_read(first, first, ReadPurpose::Value)
            .err()
            .map(Failure::message),
        Some("constant execution storage limit exceeded")
    );
    assert!(!machine.memory.ordinary_static_types.contains(&first));
    machine
        .check_static_value_read(second, second, ReadPurpose::Value)
        .map_err(Failure::message)?;
    Ok(())
}
