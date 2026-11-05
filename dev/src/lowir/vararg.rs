//! Variadic instructions retain typed pointer/result and enclosing arity checks.
use super::{
    program::{Function, Instruction, Literal, Operand, Operation, Program, ResultId, SignatureId},
    vocabulary::Type,
};
pub(super) fn pointer(
    program: &Program,
    function: &Function,
    signature: SignatureId,
    value: Operand,
) -> Result<(), &'static str> {
    let valid = match value {
        Operand::Temporary(id) => function.temporaries.get(id.0 as usize) == Some(&Type::Ptr),
        Operand::Parameter(index) => {
            program.parameter_type(signature, index as usize)?
                == super::boundary::Type::Scalar(Type::Ptr)
        }
        Operand::Slot(id) => function.slots.get(id.0 as usize).is_some(),
        Operand::Global(id) => program.globals.get(id.0 as usize).is_some(),
        Operand::Symbol(id) => program.symbols.get(id.0 as usize).is_some(),
        Operand::Literal(id) => matches!(function.literals.get(id.0 as usize), Some(Literal::Null)),
        Operand::ObjectTemporary(_) => false,
    };
    if valid {
        Ok(())
    } else {
        Err("variadic list requires its typed pointer operand")
    }
}
pub(super) fn scalar(ty: Type) -> bool {
    matches!(
        ty,
        Type::I32 | Type::U32 | Type::I64 | Type::F64 | Type::Ptr
    )
}
pub(super) fn instruction(
    program: &Program,
    function: &Function,
    signature: SignatureId,
    instruction: &Instruction,
) -> Result<(), &'static str> {
    let list = match instruction.operation {
        Operation::VaStart { list } => {
            if !program.variadic(signature)? || instruction.result.is_some() {
                return Err("va_start requires a genuine variadic function and no result");
            }
            list
        }
        Operation::VaArg { ty, list } => {
            if !scalar(ty)
                || !matches!(instruction.result,Some(ResultId::Scalar(id)) if function.temporaries.get(id.0 as usize)==Some(&ty))
            {
                return Err("va_arg requires its exact supported scalar result");
            }
            list
        }
        _ => return Ok(()),
    };
    pointer(program, function, signature, list)
}
