//! Zero-count control flow cannot reach external memory, and ZST shims have no loop.
use super::*;
use crate::{
    lowir::program::{MemorySpan, Operation, ResultId},
    support::intrinsics::{ConstPermission, Intrinsic, Safety},
};
#[test]
fn counted_copy_shim_has_unsigned_guard_and_one_typed_body_transfer() -> Result<(), &'static str> {
    for representation in [
        Representation::Scalar(Type::U32),
        Representation::Object(MemorySpan {
            bytes: 24,
            align: 8,
        }),
    ] {
        let body = Context::counted_memory_body(representation)?;
        assert_eq!(body.blocks.len(), 4);
        assert_eq!(body.slots.len(), 1);
        let (header, condition, yes, no) = body
            .blocks
            .iter()
            .enumerate()
            .find_map(|(i, b)| match b.terminator {
                Terminator::Branch { condition, yes, no } => Some((i, condition, yes, no)),
                _ => None,
            })
            .ok_or("missing counted guard")?;
        let compare = body
            .instructions
            .iter()
            .find(|i| {
                matches!(
                    i.operation,
                    Operation::Compare {
                        predicate: Predicate::Ult,
                        left: Value { ty: Type::I64, .. },
                        right: Operand::Parameter(2)
                    }
                )
            })
            .ok_or("missing unsigned count comparison")?;
        let Some(ResultId::Scalar(result)) = compare.result else {
            return Err("guard result");
        };
        assert!(matches!(condition,Operand::Temporary(id) if id==result));
        assert!(
            matches!(body.blocks[yes.0 as usize].terminator,Terminator::Jump(id) if id.0 as usize==header)
        );
        assert!(matches!(
            body.blocks[no.0 as usize].terminator,
            Terminator::Return(None)
        ));
        let mut transfers = 0;
        for (id, block) in body.blocks.iter().enumerate() {
            for instruction in &body.instructions
                [block.instructions.start as usize..block.instructions.end as usize]
            {
                let external = match instruction.operation {
                    Operation::CopyObject { .. } => true,
                    Operation::Load { address, .. } | Operation::Store { address, .. } => {
                        !matches!(address, Operand::Slot(_))
                    }
                    _ => false,
                };
                if external {
                    assert_eq!(id, yes.0 as usize);
                    transfers += 1;
                }
            }
        }
        assert_eq!(
            transfers,
            if matches!(representation, Representation::Object(_)) {
                1
            } else {
                2
            }
        );
    }
    Ok(())
}
#[test]
fn counted_zst_shim_is_effect_free_after_caller_operand_evaluation() -> Result<(), &'static str> {
    let body = Context::counted_memory_body(Representation::Scalar(Type::Void))?;
    assert!(body.instructions.is_empty());
    assert!(body.slots.is_empty());
    assert_eq!(body.blocks.len(), 1);
    assert!(matches!(
        body.blocks[0].terminator,
        Terminator::Return(None)
    ));
    assert_eq!(Intrinsic::CopyNonoverlapping.index(), 19);
    assert!(
        Intrinsic::CopyNonoverlapping.is_memory() && Intrinsic::CopyNonoverlapping.is_pointer()
    );
    assert_eq!(
        Intrinsic::CopyNonoverlapping.descriptor().safety,
        Safety::Unsafe
    );
    assert_eq!(
        Intrinsic::CopyNonoverlapping.descriptor().constant,
        ConstPermission::Allowed
    );
    Ok(())
}
