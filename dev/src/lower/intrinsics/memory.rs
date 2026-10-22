//! Ordinary pointer transfers preserve selected layouts and owned ABI snapshots.
#[cfg(test)]
#[path = "../../../tests/invariants/transfer_shapes.rs"]
mod tests;
use crate::{
    lower::context::{Context, Representation},
    lowir::{
        builder::Builder,
        program::{Function, Operand, Terminator, Value},
        vocabulary::Type,
    },
    support::intrinsics::Intrinsic,
    types::TypeId,
};
impl Context<'_> {
    pub(super) fn memory_type(&mut self, ty: TypeId) -> Result<Representation, &'static str> {
        self.layout(ty)?;
        self.representation(ty)
    }
    pub(super) fn memory_body(
        intrinsic: Intrinsic,
        representation: Representation,
    ) -> Result<Function, &'static str> {
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let writing = intrinsic.writes_pointer();
        if !intrinsic.is_memory() {
            return Err("invalid pointer memory shim identity");
        }
        let result = match representation {
            Representation::Scalar(Type::Void) => None,
            Representation::Scalar(ty) if writing => {
                builder.store(
                    Value {
                        ty,
                        operand: Operand::Parameter(1),
                    },
                    Operand::Parameter(0),
                )?;
                None
            }
            Representation::Scalar(ty) => Some(builder.load(ty, Operand::Parameter(0))?),
            Representation::PointerObject(_) if writing => {
                builder.store(
                    Value {
                        ty: Type::Ptr,
                        operand: Operand::Parameter(1),
                    },
                    Operand::Parameter(0),
                )?;
                None
            }
            Representation::PointerObject(_) => {
                Some(builder.load(Type::Ptr, Operand::Parameter(0))?)
            }
            Representation::Object(span) => {
                // The ordinary Rust ABI supplies a fresh result destination or
                // an owned argument snapshot, so these transfers cannot overlap.
                builder.copy_object(
                    if intrinsic.is_unaligned() {
                        crate::lowir::program::MemorySpan { align: 1, ..span }
                    } else {
                        span
                    },
                    Value {
                        ty: Type::Ptr,
                        operand: Operand::Parameter(1),
                    },
                    Value {
                        ty: Type::Ptr,
                        operand: Operand::Parameter(0),
                    },
                )?;
                None
            }
        };
        builder.terminate(Terminator::Return(result))?;
        builder.finish()
    }
}
