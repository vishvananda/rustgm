//! Ordinary shim transfers use exact typed spans, ABI positions and access flavor.
use super::*;
use crate::lowir::program::{Access, MemorySpan, Operation};
#[test]
fn pointer_memory_shims_preserve_aggregate_span_and_hidden_result_positions()
-> Result<(), &'static str> {
    let span = MemorySpan {
        bytes: 16,
        align: 8,
    };
    for intrinsic in [Intrinsic::Read, Intrinsic::Write] {
        let body = Context::memory_body(intrinsic, Representation::Object(span))?;
        assert_eq!(body.instructions.len(), 1);
        let Operation::CopyObject {
            span: id,
            source,
            destination,
        } = body.instructions[0].operation
        else {
            return Err("copy operation");
        };
        assert_eq!(body.spans[id.0 as usize], span);
        assert!(matches!(source, Operand::Parameter(1)));
        assert!(matches!(destination, Operand::Parameter(0)));
    }
    Ok(())
}
#[test]
fn pointer_memory_scalar_shims_use_ordinary_access_and_zst_has_no_transfer()
-> Result<(), &'static str> {
    let read = Context::memory_body(Intrinsic::Read, Representation::Scalar(Type::U32))?;
    assert!(matches!(
        read.instructions[0].operation,
        Operation::Load {
            access: Access::Ordinary,
            ..
        }
    ));
    let write = Context::memory_body(Intrinsic::Write, Representation::Scalar(Type::U32))?;
    assert!(matches!(
        write.instructions[0].operation,
        Operation::Store {
            access: Access::Ordinary,
            ..
        }
    ));
    for intrinsic in [Intrinsic::Read, Intrinsic::Write] {
        let body = Context::memory_body(intrinsic, Representation::Scalar(Type::Void))?;
        assert!(body.instructions.is_empty());
        assert_eq!(
            intrinsic.descriptor().safety,
            crate::support::intrinsics::Safety::Unsafe
        );
        assert_eq!(
            intrinsic.descriptor().constant,
            crate::support::intrinsics::ConstPermission::Allowed
        );
    }
    assert_eq!(Intrinsic::Read.index(), 13);
    assert_eq!(Intrinsic::Write.index(), 14);
    Ok(())
}

#[test]
fn unaligned_transfer_spans_do_not_claim_external_pointer_alignment() -> Result<(), &'static str> {
    for intrinsic in [Intrinsic::ReadUnaligned, Intrinsic::WriteUnaligned] {
        let span = MemorySpan {
            bytes: 24,
            align: 8,
        };
        let body = Context::memory_body(intrinsic, Representation::Object(span))?;
        assert_eq!(body.instructions.len(), 1);
        let Operation::CopyObject {
            span: id,
            source,
            destination,
        } = body.instructions[0].operation
        else {
            return Err("unaligned copy");
        };
        assert_eq!(body.spans[id.0 as usize], MemorySpan { align: 1, ..span });
        assert!(matches!(source, Operand::Parameter(1)));
        assert!(matches!(destination, Operand::Parameter(0)));
        let zst = Context::memory_body(intrinsic, Representation::Scalar(Type::Void))?;
        assert!(zst.instructions.is_empty());
        assert!(intrinsic.is_memory() && intrinsic.is_pointer() && intrinsic.is_unaligned());
        assert!(!intrinsic.is_atomic() && !intrinsic.is_volatile());
    }
    assert_eq!(Intrinsic::ReadUnaligned.index(), 17);
    assert_eq!(Intrinsic::WriteUnaligned.index(), 18);
    Ok(())
}
