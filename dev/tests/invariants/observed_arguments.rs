//! Complete argument rows avoid repeated graph traversal without sharing caller proofs.
use super::*;
use crate::types::{ItemId, Kind, TypeId, arguments::Argument, instance::Instance};

fn callable(
    facts: &mut Declarations,
    width: usize,
    region: RegionId,
) -> Result<CallerInstance, &'static str> {
    let reference = facts.types.intern(Kind::Reference {
        region,
        mutable: false,
        referent: TypeId::UNIT,
    })?;
    let fields = facts.types.intern_list(&vec![reference; width])?;
    let tuple = facts.types.intern(Kind::Tuple(fields))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(tuple)])?;
    let parameters = facts.types.intern_list(&[])?;
    let signature = facts.types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let ty = facts.types.intern(Kind::FunctionItem {
        item: ItemId(0),
        arguments,
        signature,
    })?;
    Ok(CallerInstance::Callable(Instance::new(&facts.types, ty)?))
}
fn facts() -> Result<Declarations, Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve("fn main(){}", true)?;
    let mut facts = Declarations::new(&report)?;
    facts.source_computations = Some(Box::new(super::super::Computations(
        super::super::catalog::Catalog::new()?,
    )));
    Ok(facts)
}

#[test]
fn one_observation_per_full_instance_is_independent_of_context_count()
-> Result<(), Box<dyn std::error::Error>> {
    for width in [8, 32, 128] {
        let mut measurements = Vec::new();
        for contexts in [8, 32, 128] {
            let mut facts = facts()?;
            let enclosing = callable(&mut facts, width, RegionId(7))?;
            let before = facts.body_work.canonical_steps;
            prepare(&mut facts, enclosing, 0)?;
            let graph = facts.body_work.canonical_steps - before;
            let retained = facts.constant_work.source_context_bytes;
            for _ in 0..contexts {
                let before = facts.body_work.canonical_steps;
                prepare(&mut facts, enclosing, 0)?;
                assert_eq!(facts.body_work.canonical_steps - before, 1);
                assert_eq!(facts.constant_work.source_context_bytes, retained);
                assert_eq!(regions(&facts, enclosing)?, [RegionId(7)]);
                // Every receiving namespace still checks its own fresh range.
                let mut transport = crate::types::lifetime_proof::OwnedTransport::new(
                    std::slice::from_ref(&(1..2)),
                    &[],
                    0,
                    30,
                )?;
                transport.check_actuals(regions(&facts, enclosing)?)?;
                let mut overlapping = crate::types::lifetime_proof::OwnedTransport::new(
                    std::slice::from_ref(&(1..2)),
                    &[],
                    0,
                    7,
                )?;
                assert!(
                    overlapping
                        .check_actuals(regions(&facts, enclosing)?)
                        .is_err()
                );
            }
            measurements.push((graph, retained));
            let other = callable(&mut facts, width, RegionId(8))?;
            assert!(regions(&facts, other).is_err());
            prepare(&mut facts, other, 0)?;
            assert_eq!(regions(&facts, other)?, [RegionId(8)]);
            assert_eq!(regions(&facts, enclosing)?, [RegionId(7)]);
            assert_eq!(
                facts
                    .source_computations
                    .as_deref()
                    .ok_or("catalog")?
                    .0
                    .namespaces
                    .as_ref()
                    .ok_or("namespaces")?
                    .observations
                    .len(),
                2
            );
        }
        assert!(measurements.iter().all(|row| *row == measurements[0]));
        assert!(measurements[0].0 < width * 8 + 64);
        eprintln!(
            "observation width={width} graph={} retained={}",
            measurements[0].0, measurements[0].1
        );
    }
    Ok(())
}

#[test]
fn incomplete_or_foreign_catalog_observations_never_reuse_a_row()
-> Result<(), Box<dyn std::error::Error>> {
    let mut left = facts()?;
    let enclosing = callable(&mut left, 8, RegionId(7))?;
    prepare(&mut left, enclosing, 0)?;
    let mut right = facts()?;
    let other = callable(&mut right, 8, RegionId(9))?;
    assert_eq!(enclosing, other); // Same local numeric identity, different Store.
    assert!(regions(&right, other).is_err());
    prepare(&mut right, other, 0)?;
    assert_eq!(regions(&right, other)?, [RegionId(9)]);
    assert_eq!(regions(&left, enclosing)?, [RegionId(7)]);
    let pending = callable(&mut left, 32, RegionId(10))?;
    assert!(prepare(&mut left, pending, 64 * 1024 * 1024).is_err());
    assert!(regions(&left, pending).is_err());
    prepare(&mut left, pending, 0)?;
    assert_eq!(regions(&left, pending)?, [RegionId(10)]);
    // Reserve the complete logical catalog budget before an unobserved request.
    let unobserved = callable(&mut right, 32, RegionId(10))?;
    right
        .source_computations
        .as_deref_mut()
        .ok_or("catalog")?
        .0
        .namespaces
        .as_deref_mut()
        .ok_or("namespaces")?
        .bytes = 64 * 1024 * 1024 - 1024;
    let before = right.body_work.canonical_steps;
    assert!(prepare(&mut right, unobserved, 0).is_err());
    assert!(right.body_work.canonical_steps > before + 1);
    assert!(regions(&right, unobserved).is_err());
    assert!(prepare(&mut right, CallerInstance::Original, 0).is_err());
    Ok(())
}
