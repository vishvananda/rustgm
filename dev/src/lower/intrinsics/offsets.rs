//! Runtime pointer offsets use the existing typed byte index, with no memory access.
#[cfg(test)]
#[path = "../../../tests/invariants/offset_shapes.rs"]
mod tests;
use crate::{
    lower::context::Context,
    lowir::{
        builder::Builder,
        program::{Function, Operand, Terminator, Value},
        vocabulary::{Binary, Type},
    },
    types::{Kind, instance::Instance},
};
impl Context<'_> {
    pub(super) fn pointer_offset_body(
        &mut self,
        instance: Instance,
    ) -> Result<Function, &'static str> {
        let Some(Kind::FunctionItem { signature, .. }) = self.types.kind(instance.ty()) else {
            return Err("pointer offset shim lacks callable");
        };
        let Some(Kind::Function { parameters, .. }) = self.types.kind(signature) else {
            return Err("pointer offset shim lacks signature");
        };
        let Some(Kind::Pointer { pointee, .. }) = self
            .types
            .list(parameters)
            .and_then(|inputs| inputs.first())
            .and_then(|&ty| self.types.kind(ty))
        else {
            return Err("pointer offset shim lacks pointee");
        };
        let size = self.layout(pointee)?.size;
        Self::offset_body(size)
    }
    fn offset_body(size: u64) -> Result<Function, &'static str> {
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let base = Value {
            ty: Type::Ptr,
            operand: Operand::Parameter(0),
        };
        let count = Value {
            ty: Type::I64,
            operand: Operand::Parameter(1),
        };
        let result = Self::wrapping_offset_value(&mut builder, base, count, size)?;
        builder.terminate(Terminator::Return(Some(result)))?;
        builder.finish()
    }
    /// `LowIR` integer multiply and byte indexing preserve the complete signed word.
    pub(in crate::lower) fn wrapping_offset_value(
        builder: &mut Builder,
        base: Value,
        count: Value,
        size: u64,
    ) -> Result<Value, &'static str> {
        if base.ty != Type::Ptr || count.ty != Type::I64 {
            return Err("pointer byte index requires a pointer and a signed machine word");
        }
        if size == 0 {
            return Ok(base);
        }
        let stride = builder.integer(Type::I64, u128::from(size))?;
        let displacement = builder.binary(Binary::Mul, count, stride)?;
        builder.index(Type::U8, base, displacement)
    }
}
