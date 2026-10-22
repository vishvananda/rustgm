//! Address-only copy validation must not grant ordinary memory-access authority.
use super::{Constant, Failure, Intrinsic, Machine};
use crate::{
    sema::{
        self,
        model::Scalar,
        values::{Evaluator, Reader, WritePermission},
    },
    test_source,
    types::{Kind, Primitive, TypeId},
};
#[test]
fn counted_parent_projection_preserves_each_pointer_value_permission() -> Result<(), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let element = TypeId::primitive(Primitive::U8);
    let array = facts.types.intern(Kind::Array {
        element,
        length: 16,
    })?;
    let slice = facts.types.intern(Kind::Slice(element))?;
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let root = evaluator
        .allocation(None, array)
        .map_err(Failure::message)?;
    let first = evaluator
        .subsequence(root, 2, 14, slice)
        .map_err(Failure::message)?;
    let second = evaluator
        .subsequence(first, 3, 4, slice)
        .map_err(Failure::message)?;
    let field = evaluator
        .projection(second, 1, element)
        .map_err(Failure::message)?;
    let reader = Reader::new(evaluator.facts());
    let (writable, index) = field.projection_parent(reader)?;
    let (shared, other) = field.restrict_shared().projection_parent(reader)?;
    assert_eq!((index, other), (6, 6));
    assert_eq!(writable.location(), root.location());
    assert_eq!(shared.location(), root.location());
    assert_eq!(writable.write_permission(), Some(WritePermission::Writable));
    assert_eq!(
        shared.write_permission(),
        Some(WritePermission::SharedDerived)
    );
    assert_eq!(root.write_permission(), Some(WritePermission::Writable));
    assert!(writable.metadata().is_none());
    assert!(shared.metadata().is_none());
    assert!(root.projection_parent(reader).is_err());
    assert!(second.projection_parent(reader).is_err());
    Ok(())
}
#[test]
fn counted_zero_access_does_not_weaken_ordinary_expired_reads_or_writes() -> Result<(), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(
        evaluator,
        std::iter::empty(),
        super::super::Limits::default(),
    )?;
    let ty = TypeId::primitive(Primitive::U32);
    let value = Constant::scalar(Scalar::Integer(7));
    let mut allocations = Vec::new();
    let source = machine
        .allocate(&mut allocations, Some(value), ty)
        .map_err(Failure::message)?;
    let destination = machine
        .allocate(&mut allocations, Some(value), ty)
        .map_err(Failure::message)?;
    machine.memory.expire(&allocations);
    let zero = Constant::scalar(Scalar::Integer(0));
    let one = Constant::scalar(Scalar::Integer(1));
    assert!(
        machine
            .counted_copy(
                Intrinsic::CopyNonoverlapping,
                ty,
                &[source, destination.restrict_shared(), zero]
            )
            .map_err(Failure::message)?
            .is_some()
    );
    assert_eq!(machine.slots, 0);
    assert_eq!(
        machine
            .counted_copy(
                Intrinsic::CopyNonoverlapping,
                ty,
                &[source, destination, one]
            )
            .err()
            .map(Failure::message),
        Some("constant pointer refers to expired storage")
    );
    let span = machine
        .transfer_span(source, ty, 0, 0, 9)
        .map_err(Failure::message)?;
    assert_eq!(
        machine.read(span.place()).err().map(Failure::message),
        Some("constant pointer refers to expired storage")
    );
    assert_eq!(
        machine
            .check_raw_write(span.place())
            .err()
            .map(Failure::message),
        Some("constant pointer refers to expired storage")
    );
    assert_eq!(source.write_permission(), Some(WritePermission::Writable));
    // Span checks succeed here; the copy helper must restore retention on an
    // authority error after both proofs have already been retained.
    let live_source = machine
        .allocate(&mut allocations, Some(value), ty)
        .map_err(Failure::message)?;
    let live_destination = machine
        .allocate(&mut allocations, Some(value), ty)
        .map_err(Failure::message)?;
    assert_eq!(
        machine
            .counted_copy(
                Intrinsic::CopyNonoverlapping,
                ty,
                &[live_source, live_destination.restrict_shared(), one]
            )
            .err()
            .map(Failure::message),
        Some("constant raw pointer write is derived from a shared reference")
    );
    assert_eq!(machine.slots, 0);
    machine.limits.slots = machine.memory.retained_cells();
    assert_eq!(
        machine
            .counted_copy(
                Intrinsic::CopyNonoverlapping,
                ty,
                &[source, destination, zero]
            )
            .err()
            .map(Failure::message),
        Some("constant execution storage limit exceeded")
    );
    Ok(())
}
#[test]
fn counted_small_snapshot_does_not_expand_a_large_compressed_source() -> Result<(), String> {
    let mut baseline = None;
    for length in [256, 1 << 40] {
        let report = test_source::resolve("", true)?;
        let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
        let ty = TypeId::primitive(Primitive::U32);
        let array = facts.types.intern(Kind::Array {
            element: ty,
            length,
        })?;
        let mut evaluator = Evaluator::new(&report, &mut facts);
        let value = Constant::scalar(Scalar::Integer(37));
        let repeated = evaluator.repeat(value, length).map_err(Failure::message)?;
        let mut machine = Machine::new(
            evaluator,
            std::iter::empty(),
            super::super::Limits::default(),
        )?;
        let mut allocations = Vec::new();
        let root = machine
            .allocate(&mut allocations, Some(repeated), array)
            .map_err(Failure::message)?;
        let source = machine
            .evaluator
            .projection(root, length - 1, ty)
            .map_err(Failure::message)?;
        let destination = machine
            .allocate(&mut allocations, None, ty)
            .map_err(Failure::message)?;
        let before = machine.work();
        machine
            .counted_copy(
                Intrinsic::CopyNonoverlapping,
                ty,
                &[source, destination, Constant::scalar(Scalar::Integer(1))],
            )
            .map_err(Failure::message)?
            .ok_or("copy suspended")?;
        let after = machine.work();
        let work = (
            after.steps - before.steps,
            after.peak_slots - before.peak_slots,
        );
        println!(
            "compressed counted source length={length} steps={} slots={}",
            work.0, work.1
        );
        if let Some(expected) = baseline {
            assert_eq!(work, expected);
        } else {
            baseline = Some(work);
        }
        assert!(
            matches!(Reader::new(machine.evaluator.facts()).view(repeated)?, crate::sema::values::View::Repeat { length: actual, .. } if actual == length)
        );
        let span = machine
            .transfer_span(destination, ty, 1, 4, 9)
            .map_err(Failure::message)?;
        assert_eq!(
            machine
                .read(span.place())
                .map_err(Failure::message)?
                .ok_or("read suspended")?
                .as_scalar()
                .map_err(Failure::message)?,
            Scalar::Integer(37)
        );
    }
    Ok(())
}
#[test]
fn counted_span_work_keeps_count_independent_of_ancestry_and_layout_width() -> Result<(), String> {
    counted_work_grid(Intrinsic::CopyNonoverlapping)
}
#[test]
fn overlapping_span_work_reuses_the_same_bounded_storage_proofs() -> Result<(), String> {
    counted_work_grid(Intrinsic::Copy)
}
fn counted_work_grid(intrinsic: Intrinsic) -> Result<(), String> {
    let mut count_deltas = None;
    for depth in [8, 32, 128] {
        for width in [1, 32, 128] {
            let mut baseline = Vec::new();
            for noise in [0, 1024] {
                let mut rows = Vec::new();
                for count in [8, 32, 128] {
                    let work = counted_work(depth, width, count, noise, intrinsic)?;
                    println!(
                        "{} depth={depth} width={width} count={count} noise={noise} steps={} slots={}",
                        intrinsic.name(),
                        work.0,
                        work.1
                    );
                    rows.push(work);
                }
                if noise == 0 {
                    baseline = rows;
                } else {
                    assert_eq!(rows, baseline);
                }
            }
            let delta = [
                (baseline[1].0 - baseline[0].0, baseline[1].1 - baseline[0].1),
                (baseline[2].0 - baseline[1].0, baseline[2].1 - baseline[1].1),
            ];
            if let Some(expected) = count_deltas {
                assert_eq!(delta, expected);
            } else {
                count_deltas = Some(delta);
            }
        }
    }
    Ok(())
}
fn counted_work(
    depth: usize,
    width: usize,
    count: u64,
    noise: u64,
    intrinsic: Intrinsic,
) -> Result<(usize, usize), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let scalar = TypeId::primitive(Primitive::U32);
    for length in 1..=noise {
        facts.types.intern(Kind::Array {
            element: scalar,
            length,
        })?;
    }
    let fields = facts.types.intern_list(&vec![scalar; width])?;
    let element = facts.types.intern(Kind::Tuple(fields))?;
    let array = facts.types.intern(Kind::Array {
        element,
        length: 256,
    })?;
    let mut types = vec![array];
    for _ in 0..depth {
        let child = *types.last().ok_or("missing wrapper type")?;
        let fields = facts.types.intern_list(&[child])?;
        types.push(facts.types.intern(Kind::Tuple(fields))?);
    }
    let mut evaluator = Evaluator::new(&report, &mut facts);
    let values = counted_values(&mut evaluator, width, depth)?;
    let mut machine = Machine::new(
        evaluator,
        std::iter::empty(),
        super::super::Limits::default(),
    )?;
    let mut allocations = Vec::new();
    let mut addresses = Vec::new();
    for value in values {
        let mut address = machine
            .allocate(
                &mut allocations,
                Some(value),
                *types.last().ok_or("wrapper type")?,
            )
            .map_err(Failure::message)?;
        for &child in types.iter().rev().skip(1) {
            address = machine
                .evaluator
                .projection(address, 0, child)
                .map_err(Failure::message)?;
        }
        addresses.push(address);
    }
    let source = machine
        .evaluator
        .projection(addresses[0], 0, element)
        .map_err(Failure::message)?;
    let destination = machine
        .evaluator
        .projection(addresses[1], 0, element)
        .map_err(Failure::message)?;
    let before = machine.work();
    machine
        .counted_copy(
            intrinsic,
            element,
            &[
                source,
                destination,
                Constant::scalar(Scalar::Integer(u128::from(count))),
            ],
        )
        .map_err(Failure::message)?
        .ok_or("copy suspended")?;
    let after = machine.work();
    counted_result(&mut machine, addresses[1], element, width, count)?;
    Ok((
        after.steps - before.steps,
        after.peak_slots - before.peak_slots,
    ))
}
fn counted_values(
    evaluator: &mut Evaluator<'_>,
    width: usize,
    depth: usize,
) -> Result<Vec<Constant>, String> {
    let mut values = Vec::new();
    for bits in [7, 0] {
        let value = evaluator
            .fields(&vec![Constant::scalar(Scalar::Integer(bits)); width], None)
            .map_err(Failure::message)?;
        let mut value = evaluator.repeat(value, 256).map_err(Failure::message)?;
        for _ in 0..depth {
            value = evaluator.fields(&[value], None).map_err(Failure::message)?;
        }
        values.push(value);
    }
    Ok(values)
}
fn counted_result(
    machine: &mut Machine<'_, '_>,
    destination: Constant,
    element: TypeId,
    width: usize,
    count: u64,
) -> Result<(), String> {
    for (index, expected) in [(count - 1, 7), (count, 0)] {
        let address = machine
            .evaluator
            .projection(destination, index, element)
            .map_err(Failure::message)?;
        let span = machine
            .transfer_span(
                address,
                element,
                1,
                u64::try_from(width).map_err(|e| e.to_string())? * 4,
                9,
            )
            .map_err(Failure::message)?;
        let value = machine
            .read(span.place())
            .map_err(Failure::message)?
            .ok_or("read suspended")?;
        assert_eq!(
            machine
                .evaluator
                .field(value, 0)
                .map_err(Failure::message)?
                .as_scalar()
                .map_err(Failure::message)?,
            Scalar::Integer(expected)
        );
    }
    Ok(())
}
