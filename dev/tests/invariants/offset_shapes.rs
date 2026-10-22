//! Arithmetic shims contain no memory access, loop or pointer reconstruction.
use super::*;
use crate::{
    lowir::program::Operation,
    support::intrinsics::{ConstPermission, Intrinsic, Safety},
};
#[test]
fn offset_shims_only_scale_and_index_the_incoming_pointer() -> Result<(), &'static str> {
    for size in [0, 1, 4, 24] {
        let body = Context::offset_body(size)?;
        assert_eq!(body.blocks.len(), 1);
        assert!(body.slots.is_empty() && body.spans.is_empty());
        assert_eq!(body.instructions.len(), if size == 0 { 0 } else { 2 });
        if size == 0 {
            assert!(matches!(
                body.blocks[0].terminator,
                Terminator::Return(Some(Value {
                    ty: Type::Ptr,
                    operand: Operand::Parameter(0)
                }))
            ));
        } else {
            assert!(matches!(
                body.instructions[0].operation,
                Operation::Binary {
                    op: Binary::Mul,
                    ..
                }
            ));
            assert!(matches!(
                body.instructions[1].operation,
                Operation::Index {
                    element: Type::U8,
                    base: Operand::Parameter(0),
                    ..
                }
            ));
        }
    }
    for (kind, index) in [
        (Intrinsic::ConstPointerAdd, 22),
        (Intrinsic::MutPointerAdd, 23),
    ] {
        assert_eq!(kind.index(), index);
        assert!(kind.is_pointer_offset() && kind.is_pointer());
        assert!(!kind.is_pointer_function() && !kind.is_memory() && !kind.is_atomic());
        assert_eq!(kind.descriptor().constant, ConstPermission::Allowed);
        assert_eq!(kind.descriptor().safety, Safety::Unsafe);
    }
    Ok(())
}
