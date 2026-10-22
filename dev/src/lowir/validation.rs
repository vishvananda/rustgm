//! Object identities and scalar identities remain disjoint at each physical boundary.
use super::{
    boundary::{self, ObjectValue},
    program::{
        BoundaryArgument, CallArguments, Function, Instruction, Operand, Operation, Program,
        ResultId, SignatureId, Storage, Value,
    },
};
pub(super) fn object_operand(
    program: &Program,
    function: &Function,
    signature: Option<SignatureId>,
    value: ObjectValue,
) -> Result<(), &'static str> {
    let span = match value.operand {
        Operand::Slot(id) => match function.slots.get(id.0 as usize) {
            Some(Storage::Object(span)) => function.spans.get(span.0 as usize).copied(),
            _ => None,
        },
        Operand::ObjectTemporary(id) => function
            .objects
            .as_ref()
            .and_then(|objects| objects.temporaries.get(id.0 as usize))
            .and_then(|span| function.spans.get(span.0 as usize))
            .copied(),
        Operand::Global(id) => program.globals.get(id.0 as usize).map(|global| global.span),
        Operand::Parameter(index) => {
            if let Some(signature) = signature {
                match program
                    .object_boundary(signature)?
                    .parameters
                    .get(index as usize)
                {
                    Some(boundary::Type::Object(object)) => Some(object.span),
                    _ => None,
                }
            } else {
                Some(value.span)
            }
        }
        _ => None,
    };
    if span != Some(value.span) {
        return Err("invalid or differently spanned LowIR object operand");
    }
    Ok(())
}
pub(super) fn scalar_operand(value: Operand) -> Result<(), &'static str> {
    if matches!(value, Operand::ObjectTemporary(_)) {
        Err("object temporary used as scalar operand")
    } else {
        Ok(())
    }
}
pub(super) fn call(
    program: &Program,
    function: &Function,
    signature: SignatureId,
    instruction: &Instruction,
) -> Result<(), &'static str> {
    let Operation::Call {
        signature: target,
        arguments,
        callee,
    } = instruction.operation
    else {
        return Err("expected LowIR call");
    };
    scalar_operand(callee)?;
    let result = match arguments {
        CallArguments::Scalar(range) => {
            let target = program.scalar_signature(target)?;
            let arguments = function
                .arguments
                .get(range.start as usize..range.end as usize)
                .ok_or("invalid scalar call arguments")?;
            let expected = program.parameters(match instruction.operation {
                Operation::Call { signature, .. } => signature,
                _ => return Err("expected call"),
            })?;
            if arguments.len() < expected.len()
                || (!target.variadic && arguments.len() != expected.len())
            {
                return Err("scalar call arity disagrees with signature");
            }
            for (value, &ty) in arguments.iter().zip(expected) {
                if value.ty != ty {
                    return Err("scalar call argument type mismatch");
                }
                scalar_operand(value.operand)?;
            }
            for value in &arguments[expected.len()..] {
                variadic_scalar(program, function, signature, *value)?;
            }
            boundary::Type::Scalar(target.result)
        }
        CallArguments::Object(range) => {
            let fixed = program.parameter_count(target)?;
            let variadic = program.variadic(target)?;
            let arguments = function
                .objects
                .as_ref()
                .and_then(|objects| {
                    objects
                        .arguments
                        .get(range.start as usize..range.end as usize)
                })
                .ok_or("invalid object call arguments")?;
            if arguments.len() < fixed || (!variadic && arguments.len() != fixed) {
                return Err("object call arity disagrees with signature");
            }
            for (index, &value) in arguments.iter().enumerate() {
                let expected = if index < fixed {
                    program.parameter_type(target, index)?
                } else {
                    variadic_argument_type(program, function, signature, value)?
                };
                match (value, expected) {
                    (BoundaryArgument::Scalar(value), boundary::Type::Scalar(ty))
                        if value.ty == ty =>
                    {
                        scalar_operand(value.operand)?;
                    }
                    (
                        BoundaryArgument::Object {
                            operand,
                            span,
                            classes,
                        },
                        boundary::Type::Object(object),
                    ) if function.spans.get(span.0 as usize) == Some(&object.span)
                        && (index >= fixed || classes.is_none()) =>
                    {
                        object_operand(
                            program,
                            function,
                            Some(signature),
                            ObjectValue {
                                operand,
                                span: object.span,
                            },
                        )?;
                    }
                    _ => return Err("object call argument span or kind mismatch"),
                }
            }
            program.result_type(target)?
        }
    };
    call_result(program, function, signature, instruction.result, result)
}
fn call_result(
    program: &Program,
    function: &Function,
    signature: SignatureId,
    id: Option<ResultId>,
    result: boundary::Type,
) -> Result<(), &'static str> {
    match (result, id) {
        (boundary::Type::Scalar(super::vocabulary::Type::Void), None) => Ok(()),
        (boundary::Type::Scalar(ty), Some(ResultId::Scalar(id)))
            if function.temporaries.get(id.0 as usize) == Some(&ty) =>
        {
            Ok(())
        }
        (boundary::Type::Object(object), Some(ResultId::Object(id))) => object_operand(
            program,
            function,
            Some(signature),
            ObjectValue {
                operand: Operand::ObjectTemporary(id),
                span: object.span,
            },
        ),
        _ => Err("LowIR call result identity or kind differs from boundary"),
    }
}
fn variadic_argument_type(
    program: &Program,
    function: &Function,
    owner: SignatureId,
    value: BoundaryArgument,
) -> Result<boundary::Type, &'static str> {
    match value {
        BoundaryArgument::Scalar(value) => {
            variadic_scalar(program, function, owner, value)?;
            Ok(boundary::Type::Scalar(value.ty))
        }
        BoundaryArgument::Object {
            span,
            classes: Some(classes),
            ..
        } => {
            let span = *function
                .spans
                .get(span.0 as usize)
                .ok_or("missing variadic object span")?;
            let object = boundary::Object { span, classes };
            object.validate(false)?;
            Ok(boundary::Type::Object(object))
        }
        BoundaryArgument::Object { .. } => {
            Err("variadic argument lacks its explicit object classes")
        }
    }
}
fn variadic_scalar(
    program: &Program,
    function: &Function,
    owner: SignatureId,
    value: Value,
) -> Result<(), &'static str> {
    use super::vocabulary::Type;
    let actual = match value.operand {
        Operand::Temporary(id) => function.temporaries.get(id.0 as usize).copied(),
        Operand::Parameter(index) => match program.parameter_type(owner, index as usize)? {
            boundary::Type::Scalar(ty) => Some(ty),
            boundary::Type::Object(_) => None,
        },
        Operand::Slot(id) => function.slots.get(id.0 as usize).map(|_| Type::Ptr),
        Operand::Symbol(id) => program.symbols.get(id.0 as usize).map(|_| Type::Ptr),
        Operand::Global(id) => program.globals.get(id.0 as usize).map(|_| Type::Ptr),
        Operand::Literal(_) | Operand::ObjectTemporary(_) => None,
    };
    if value.ty == Type::Void || actual != Some(value.ty) {
        return Err("variadic scalar operand differs from its explicit physical type");
    }
    Ok(())
}
pub(super) fn scalar_operation(
    function: &Function,
    instruction: &Instruction,
) -> Result<(), &'static str> {
    if matches!(instruction.result, Some(ResultId::Object(_)))
        && !matches!(instruction.operation, Operation::Call { .. })
    {
        return Err("object result on scalar LowIR instruction");
    }
    let scalar = |value: Value| {
        scalar_operand(value.operand)?;
        if let Operand::Temporary(id) = value.operand
            && function.temporaries.get(id.0 as usize) != Some(&value.ty)
        {
            return Err("invalid scalar temporary type");
        }
        Ok(())
    };
    match instruction.operation {
        Operation::Address(value) => scalar_operand(value)?,
        Operation::Copy(value)
        | Operation::Unary { value, .. }
        | Operation::Convert { value, .. }
        | Operation::Store { value, .. }
        | Operation::AtomicStore { value, .. }
        | Operation::AtomicExchange { value, .. }
        | Operation::AtomicAddFetch { value, .. }
        | Operation::AtomicCompareExchange { value, .. } => scalar(value)?,
        Operation::Binary { left, right, .. } | Operation::Compare { left, right, .. } => {
            scalar(left)?;
            scalar_operand(right)?;
        }
        Operation::Index { base, offset, .. } => {
            scalar_operand(base)?;
            scalar_operand(offset)?;
        }
        _ => {}
    }
    match instruction.operation {
        Operation::Load { address, .. }
        | Operation::Store { address, .. }
        | Operation::AtomicLoad { address, .. }
        | Operation::AtomicStore { address, .. }
        | Operation::AtomicExchange { address, .. }
        | Operation::AtomicAddFetch { address, .. } => scalar_operand(address)?,
        Operation::AtomicCompareExchange {
            address, expected, ..
        } => {
            scalar_operand(address)?;
            scalar_operand(expected)?;
        }
        Operation::CopyObject { destination, .. } => scalar_operand(destination)?,
        Operation::Call { callee, .. } => scalar_operand(callee)?,
        _ => {}
    }
    Ok(())
}

pub(super) fn copy_source(
    program: &Program,
    function: &Function,
    owner: SignatureId,
    source: Operand,
    span: super::program::MemorySpan,
) -> Result<(), &'static str> {
    let object = matches!(source, Operand::ObjectTemporary(_))
        || matches!(source,Operand::Parameter(index) if program.object_boundary(owner).is_ok_and(|signature|matches!(signature.parameters.get(index as usize),Some(boundary::Type::Object(_)))));
    if object {
        object_operand(
            program,
            function,
            Some(owner),
            ObjectValue {
                operand: source,
                span,
            },
        )
    } else {
        scalar_operand(source)
    }
}
