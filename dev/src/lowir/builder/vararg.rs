//! A reached typed source operation supplies the list address and enclosing ABI.
use super::{Builder, Operation, SignatureId, Type, Value, program::Program};
impl Builder {
    /// Initialize a list in a real variadic function.
    /// # Errors
    /// Rejects non-pointer storage, fixed arity and invalid builder state.
    pub fn va_start(
        &mut self,
        program: &Program,
        signature: SignatureId,
        list: Value,
    ) -> Result<(), &'static str> {
        if list.ty != Type::Ptr || !program.variadic(signature)? {
            return Err("va_start requires a pointer in a genuine variadic function");
        }
        super::super::vararg::pointer(program, &self.function, signature, list.operand)?;
        self.emit(Operation::VaStart { list: list.operand }, None)
    }
    /// Read and advance one qualified scalar argument.
    /// # Errors
    /// Rejects non-pointer storage, unsupported scalar types and invalid state.
    pub fn va_arg(
        &mut self,
        program: &Program,
        signature: SignatureId,
        ty: Type,
        list: Value,
    ) -> Result<Value, &'static str> {
        if list.ty != Type::Ptr || !super::super::vararg::scalar(ty) {
            return Err("va_arg requires a pointer and its supported scalar result");
        }
        super::super::vararg::pointer(program, &self.function, signature, list.operand)?;
        self.value(
            ty,
            Operation::VaArg {
                ty,
                list: list.operand,
            },
        )
    }
}
