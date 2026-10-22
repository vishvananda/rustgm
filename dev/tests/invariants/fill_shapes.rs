//! The unsigned extent guard dominates every multiplication and external store.
use super::*;
use crate::{
    lowir::program::{MemorySpan, Operation},
    support::intrinsics::{ConstPermission, Intrinsic, Safety},
};
#[test]
fn byte_fill_shim_has_one_byte_store_and_no_external_read() -> Result<(), &'static str> {
    for representation in [
        Representation::Scalar(Type::U32),
        Representation::Object(MemorySpan {
            bytes: 24,
            align: 8,
        }),
    ] {
        let body = Context::fill_memory_body(representation)?;
        assert_eq!(body.blocks.len(), 6);
        let Terminator::Branch { yes, no, .. } = body.blocks[0].terminator else {
            return Err("missing overflow guard");
        };
        assert!(matches!(
            body.blocks[yes.0 as usize].terminator,
            Terminator::Unreachable
        ));
        assert!(body.instructions.iter().any(|i| matches!(
            i.operation,
            Operation::Compare {
                predicate: Predicate::Ugt,
                ..
            }
        )));
        let mut external = Vec::new();
        for (index, block) in body.blocks.iter().enumerate() {
            for instruction in &body.instructions
                [block.instructions.start as usize..block.instructions.end as usize]
            {
                match instruction.operation {
                    Operation::CopyObject { .. } => return Err("fill uses copy"),
                    Operation::Load { address, .. } if !matches!(address, Operand::Slot(_)) => {
                        return Err("fill reads external storage");
                    }
                    Operation::Store { value, address, .. }
                        if !matches!(address, Operand::Slot(_)) =>
                    {
                        assert_eq!(value.ty, Type::U8);
                        assert!(matches!(value.operand, Operand::Parameter(1)));
                        external.push(index);
                    }
                    Operation::Binary {
                        op: Binary::Mul, ..
                    } => assert_eq!(index, no.0 as usize),
                    _ => {}
                }
            }
        }
        assert_eq!(external.len(), 1);
        assert_ne!(external[0], 0);
        assert_ne!(external[0], no.0 as usize);
    }
    let zst = Context::fill_memory_body(Representation::Scalar(Type::Void))?;
    assert!(zst.instructions.is_empty() && zst.slots.is_empty());
    assert_eq!(zst.blocks.len(), 1);
    assert_eq!(Intrinsic::WriteBytes.index(), 21);
    assert!(Intrinsic::WriteBytes.is_fill() && !Intrinsic::WriteBytes.is_counted_memory());
    assert_eq!(Intrinsic::WriteBytes.descriptor().safety, Safety::Unsafe);
    assert_eq!(
        Intrinsic::WriteBytes.descriptor().constant,
        ConstPermission::Allowed
    );
    Ok(())
}
