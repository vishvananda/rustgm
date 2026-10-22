//! Shared span proofs retain permissions while overlapping moves snapshot original cells.
use super::{Constant, Failure, Intrinsic, Machine};
use crate::{
    const_eval::memory::Resolved,
    sema::{
        self,
        model::Scalar,
        values::{Evaluator, WritePermission},
    },
    test_source,
    types::{Kind, Primitive, TypeId},
};
#[test]
fn overlapping_copy_reads_the_original_cells_before_either_direction_of_stores()
-> Result<(), String> {
    for (source_index, destination_index, expected) in [
        (0, 1, [2, 2, 3, 5, 7, 13]),
        (1, 0, [3, 5, 7, 11, 11, 13]),
        (0, 0, [2, 3, 5, 7, 11, 13]),
    ] {
        let report = test_source::resolve("", true)?;
        let mut facts = sema::declarations::analyze(&report).map_err(|e| e.to_string())?;
        let ty = TypeId::primitive(Primitive::U32);
        let array = facts.types.intern(Kind::Array {
            element: ty,
            length: 6,
        })?;
        let mut evaluator = Evaluator::new(&report, &mut facts);
        let fields = [2, 3, 5, 7, 11, 13].map(|n| Constant::scalar(Scalar::Integer(n)));
        let value = evaluator.fields(&fields, None).map_err(Failure::message)?;
        let mut machine = Machine::new(
            evaluator,
            std::iter::empty(),
            super::super::Limits::default(),
        )?;
        let mut allocations = Vec::new();
        let root = machine
            .allocate(&mut allocations, Some(value), array)
            .map_err(Failure::message)?;
        let source = machine
            .evaluator
            .projection(root, source_index, ty)
            .map_err(Failure::message)?;
        let destination = machine
            .evaluator
            .projection(root, destination_index, ty)
            .map_err(Failure::message)?;
        let count = Constant::scalar(Scalar::Integer(4));
        assert_eq!(
            machine
                .counted_copy(
                    Intrinsic::CopyNonoverlapping,
                    ty,
                    &[source, destination, count]
                )
                .err()
                .map(Failure::message),
            Some("constant copy_nonoverlapping regions overlap")
        );
        assert_eq!(
            machine
                .counted_copy(
                    Intrinsic::Copy,
                    ty,
                    &[source, destination.restrict_shared(), count]
                )
                .err()
                .map(Failure::message),
            Some("constant raw pointer write is derived from a shared reference")
        );
        assert_eq!(root.write_permission(), Some(WritePermission::Writable));
        machine
            .counted_copy(Intrinsic::Copy, ty, &[source, destination, count])
            .map_err(Failure::message)?
            .ok_or("copy suspended")?;
        assert_eq!(machine.slots, 0);
        let result = machine
            .read(Resolved {
                address: root,
                ty: array,
                raw: None,
                union: None,
            })
            .map_err(Failure::message)?
            .ok_or("read suspended")?;
        for (index, n) in expected.into_iter().enumerate() {
            assert_eq!(
                machine
                    .evaluator
                    .field(result, index as u64)
                    .map_err(Failure::message)?
                    .as_scalar()
                    .map_err(Failure::message)?,
                Scalar::Integer(n)
            );
        }
        assert_eq!(
            machine
                .counted_copy(Intrinsic::Read, ty, &[source, destination, count])
                .err()
                .map(Failure::message),
            Some("constant counted copy has an invalid intrinsic identity")
        );
    }
    Ok(())
}
