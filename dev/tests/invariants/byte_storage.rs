//! One backing preserves aliases, masks and per-pointer authority after conversion.
use super::{Cell, Constant, Failure, Machine, Reader, Snapshot};
use crate::{
    const_eval::{
        Limits,
        memory::{Range, Resolved},
    },
    sema::{self, model::Scalar, values::Evaluator},
    support::intrinsics::Intrinsic,
    test_source,
    types::{Kind, Primitive, TypeId},
};
fn with_array(
    length: u64,
    slots: usize,
    check: impl FnOnce(&mut Machine<'_, '_>, TypeId) -> Result<(), Failure>,
) -> Result<(), String> {
    let report = test_source::resolve("", true)?;
    let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
    let ty = facts.types.intern(Kind::Array {
        element: TypeId::primitive(Primitive::U8),
        length,
    })?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(
        evaluator,
        std::iter::empty(),
        Limits {
            slots,
            ..Limits::default()
        },
    )?;
    check(&mut machine, ty).map_err(|error| error.message().to_owned())
}
fn scalar(value: u128) -> Constant {
    Constant::scalar(Scalar::Integer(value))
}
#[test]
fn byte_backing_clears_mask_without_typed_interpretation() -> Result<(), String> {
    with_array(4, 1 << 20, |machine, ty| {
        let mut allocations = Vec::new();
        let value = machine.evaluator.repeat(scalar(9), 4)?;
        let source = machine.allocate(&mut allocations, Some(value), ty)?;
        let target = machine.allocate(&mut allocations, Some(value), ty)?;
        let root = source.location().ok_or("source root")?;
        machine.commit_byte_snapshot(
            root,
            0,
            &Snapshot {
                bytes: vec![7, 0, 0, 0],
                initialized: vec![1],
            },
        )?;
        assert!(machine.read_cell(root, &[0])? == scalar(7));
        assert!(machine.read_cell(root, &[1]).is_err());
        assert!(
            machine
                .counted_copy(
                    Intrinsic::CopyNonoverlapping,
                    ty,
                    &[source, target, scalar(1)]
                )?
                .is_some()
        );
        let destination = target.location().ok_or("target root")?;
        assert!(machine.read_cell(destination, &[0])? == scalar(7));
        assert!(machine.read_cell(destination, &[1]).is_err());
        assert_eq!(
            machine.sequence_length(Resolved {
                address: target,
                ty,
                raw: None,
                union: None
            })?,
            4
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
#[test]
fn byte_backing_aliases_overlap_and_ordinary_range_writes() -> Result<(), String> {
    with_array(5, 1 << 20, |machine, ty| {
        let mut allocations = Vec::new();
        let value = machine.evaluator.fields(
            &[scalar(1), scalar(2), scalar(3), scalar(4), scalar(5)],
            None,
        )?;
        let address = machine.allocate(&mut allocations, Some(value), ty)?;
        let root = address.location().ok_or("root")?;
        machine.convert_byte_root(root)?;
        let snapshot = machine.snapshot_byte_root(root, 0, 4)?.ok_or("snapshot")?;
        machine.commit_byte_snapshot(root, 1, &snapshot)?;
        for (index, expected) in [1, 1, 2, 3, 4].into_iter().enumerate() {
            assert!(machine.read_cell(root, &[index as u64])? == scalar(expected));
        }
        let replacement = machine.evaluator.fields(&[scalar(11), scalar(13)], None)?;
        machine.write_range(root, &[], Range { from: 1, length: 2 }, replacement)?;
        assert!(machine.read_cell(root, &[1])? == scalar(11));
        assert!(machine.read_cell(root, &[2])? == scalar(13));
        let element = machine
            .evaluator
            .projection(address, 2, TypeId::primitive(Primitive::U8))?;
        machine.write(element, scalar(17))?;
        assert!(machine.read_cell(root, &[2])? == scalar(17));
        assert_eq!(machine.memory.bytes.roots.len(), 1);
        let &cell = machine.memory.roots.get(&root).ok_or("root cell")?;
        assert!(matches!(machine.memory.cells[cell], Cell::Bytes(_)));
        println!(
            "Constant={} Cell={} Backing={}",
            size_of::<Constant>(),
            size_of::<Cell>(),
            size_of::<super::Backing>()
        );
        Ok(())
    })
}
#[test]
fn byte_backing_keeps_pointer_permission_and_expiry_checks() -> Result<(), String> {
    with_array(4, 1 << 20, |machine, ty| {
        let mut allocations = Vec::new();
        let value = machine.evaluator.repeat(scalar(7), 4)?;
        let source = machine.allocate(&mut allocations, Some(value), ty)?;
        let target = machine.allocate(&mut allocations, Some(value), ty)?;
        machine.convert_byte_root(source.location().ok_or("root")?)?;
        assert!(
            machine
                .counted_copy(
                    Intrinsic::CopyNonoverlapping,
                    ty,
                    &[source, target.restrict_shared(), scalar(1)]
                )
                .is_err()
        );
        assert!(
            machine
                .counted_copy(
                    Intrinsic::CopyNonoverlapping,
                    ty,
                    &[source, target, scalar(1)]
                )?
                .is_some()
        );
        assert_eq!(
            target.write_permission(),
            Some(crate::sema::values::WritePermission::Writable)
        );
        machine.memory.expire(&allocations);
        assert!(
            machine
                .counted_copy(Intrinsic::Copy, ty, &[source, target, scalar(1)])
                .is_err()
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
#[test]
fn byte_backing_budget_failure_keeps_original_typed_root() -> Result<(), String> {
    with_array(4096, 32, |machine, ty| {
        let mut allocations = Vec::new();
        let value = machine.evaluator.repeat(scalar(7), 4096)?;
        let address = machine.allocate(&mut allocations, Some(value), ty)?;
        let root = address.location().ok_or("root")?;
        assert!(machine.convert_byte_root(root).is_err());
        assert!(machine.memory.bytes.roots.is_empty());
        assert!(machine.read_cell(root, &[0])? == scalar(7));
        assert_eq!(
            Reader::new(machine.evaluator.facts()).location(root)?.ty,
            ty
        );
        assert_eq!(machine.slots, 0);
        Ok(())
    })
}
