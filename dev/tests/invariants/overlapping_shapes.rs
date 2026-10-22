//! Direction guards dominate byte transfers; overlapping spans never use `CopyObject`.
use super::*;
use crate::{
    lowir::program::{MemorySpan, Operation},
    support::intrinsics::{ConstPermission, Intrinsic, Safety},
};
#[test]
fn overlapping_shim_uses_unsigned_pointer_direction_and_guarded_byte_backedges()
-> Result<(), &'static str> {
    for representation in [
        Representation::Scalar(Type::U32),
        Representation::Object(MemorySpan {
            bytes: 24,
            align: 8,
        }),
    ] {
        let body = Context::counted_memory_body(Intrinsic::Copy, representation)?;
        assert_eq!(body.blocks.len(), 9);
        assert_eq!(body.slots.len(), 1);
        assert!(body.instructions.iter().any(|i| matches!(
            i.operation,
            Operation::Compare {
                predicate: Predicate::Ugt,
                left: Value {
                    ty: Type::Ptr,
                    operand: Operand::Parameter(1)
                },
                right: Operand::Parameter(0)
            }
        )));
        let mut transfers = 0;
        let mut loops = 0;
        for block in &body.blocks {
            for instruction in &body.instructions
                [block.instructions.start as usize..block.instructions.end as usize]
            {
                assert!(!matches!(
                    instruction.operation,
                    Operation::CopyObject { .. }
                ));
                let external = match instruction.operation {
                    Operation::Load { ty: Type::U8, .. }
                    | Operation::Store {
                        value: Value { ty: Type::U8, .. },
                        ..
                    } => true,
                    Operation::Load { address, .. } | Operation::Store { address, .. } => {
                        assert!(matches!(address, Operand::Slot(_)));
                        false
                    }
                    _ => false,
                };
                if external {
                    assert!(matches!(block.terminator, Terminator::Jump(_)));
                    transfers += 1;
                }
            }
            if let Terminator::Branch { yes, no, .. } = block.terminator
                && matches!(
                    body.blocks[no.0 as usize].terminator,
                    Terminator::Return(None)
                )
                && let Terminator::Jump(header) = body.blocks[yes.0 as usize].terminator
            {
                assert!(matches!(
                    body.blocks[header.0 as usize].terminator,
                    Terminator::Branch { .. }
                ));
                loops += 1;
            }
        }
        assert_eq!(transfers, 4);
        assert_eq!(loops, 2);
        assert!(
            matches!(body.blocks[0].terminator, Terminator::Branch { yes, .. } if matches!(body.blocks[yes.0 as usize].terminator, Terminator::Return(None)))
        );
    }
    Ok(())
}
#[test]
fn overlapping_zero_size_shim_has_no_byte_count_arithmetic_or_memory() -> Result<(), &'static str> {
    let body = Context::counted_memory_body(Intrinsic::Copy, Representation::Scalar(Type::Void))?;
    assert!(body.instructions.is_empty());
    assert!(body.slots.is_empty());
    assert_eq!(body.blocks.len(), 1);
    assert_eq!(Intrinsic::Copy.index(), 20);
    assert_eq!(Intrinsic::Copy.descriptor().safety, Safety::Unsafe);
    assert_eq!(
        Intrinsic::Copy.descriptor().constant,
        ConstPermission::Allowed
    );
    Ok(())
}
