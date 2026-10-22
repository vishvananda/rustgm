//! Boundary serialization observes canonical classes and sparse object identities.
use super::{Function, Instruction, Operand, Operation, Program, SignatureId, Write, io};
use crate::lowir::{
    boundary,
    program::{BoundaryArgument, CallArguments, Signature},
    validation,
};
pub(super) fn ty(ty: boundary::Type, out: &mut impl Write) -> io::Result<()> {
    match ty {
        boundary::Type::Scalar(ty) => write!(out, "{}", ty.name()),
        boundary::Type::Object(object) => {
            write!(out, "obj<{}x{}>", object.span.bytes, object.span.align)
        }
    }
}
fn classes(value: boundary::Classes, out: &mut impl Write) -> io::Result<()> {
    match value {
        boundary::Classes::Memory => write!(out, "memory"),
        boundary::Classes::Eightbytes { first, second } => {
            write!(out, "{}", first.name())?;
            if let Some(second) = second {
                write!(out, "_{}", second.name())?;
            }
            Ok(())
        }
    }
}
pub(super) fn result(program: &Program, id: SignatureId, out: &mut impl Write) -> io::Result<()> {
    match program.signatures.get(id.0 as usize) {
        Some(Signature::Scalar(value)) => write!(out, "{}", value.result.name()),
        Some(Signature::Object(_)) => ty(
            program
                .object_boundary(id)
                .map_err(io::Error::other)?
                .result,
            out,
        ),
        None => Err(io::Error::other("invalid LowIR signature")),
    }
}
pub(super) fn result_metadata(
    program: &Program,
    id: SignatureId,
    out: &mut impl Write,
) -> io::Result<()> {
    if program.variadic(id).map_err(io::Error::other)? {
        write!(out, " [arity=variadic]")?;
    }
    if matches!(
        program.signatures.get(id.0 as usize),
        Some(Signature::Object(_))
    ) && let boundary::Type::Object(object) = program
        .object_boundary(id)
        .map_err(io::Error::other)?
        .result
        && object.span.bytes <= 16
    {
        write!(out, " [result_classes=")?;
        classes(object.classes, out)?;
        write!(out, "]")?;
    }
    Ok(())
}
pub(super) fn parameters(
    program: &Program,
    id: SignatureId,
    names: bool,
    out: &mut impl Write,
) -> io::Result<()> {
    if let Some(Signature::Scalar(_)) = program.signatures.get(id.0 as usize) {
        for (index, ty) in program
            .parameters(id)
            .map_err(io::Error::other)?
            .iter()
            .enumerate()
        {
            if index != 0 {
                write!(out, ", ")?;
            }
            if names {
                write!(out, "%p{index} : ")?;
            }
            write!(out, "{}", ty.name())?;
        }
    } else {
        for (index, &parameter) in program
            .object_boundary(id)
            .map_err(io::Error::other)?
            .parameters
            .iter()
            .enumerate()
        {
            if index != 0 {
                write!(out, ", ")?;
            }
            if names {
                write!(out, "%p{index} : ")?;
            }
            ty(parameter, out)?;
            if let boundary::Type::Object(object) = parameter
                && object.span.bytes <= 16
            {
                write!(out, " [classes=")?;
                classes(object.classes, out)?;
                write!(out, "]")?;
            }
        }
    }
    Ok(())
}
pub(super) fn call(
    program: &Program,
    function: &Function,
    owner: SignatureId,
    instruction: &Instruction,
    out: &mut impl Write,
) -> io::Result<()> {
    validation::call(program, function, owner, instruction).map_err(io::Error::other)?;
    let Operation::Call {
        signature,
        callee,
        arguments,
    } = instruction.operation
    else {
        return Err(io::Error::other("expected call"));
    };
    write!(out, "call ")?;
    result(program, signature, out)?;
    write!(out, " ")?;
    super::operand(program, function, callee, out)?;
    write!(out, "(")?;
    match arguments {
        CallArguments::Scalar(range) => {
            for (index, value) in function.arguments[range.start as usize..range.end as usize]
                .iter()
                .enumerate()
            {
                if index != 0 {
                    write!(out, ", ")?;
                }
                super::operand(program, function, value.operand, out)?;
            }
        }
        CallArguments::Object(range) => {
            let objects = function
                .objects
                .as_ref()
                .ok_or_else(|| io::Error::other("missing object call arguments"))?;
            for (index, value) in objects.arguments[range.start as usize..range.end as usize]
                .iter()
                .enumerate()
            {
                if index != 0 {
                    write!(out, ", ")?;
                }
                let operand = match *value {
                    BoundaryArgument::Scalar(value) => value.operand,
                    BoundaryArgument::Object { operand, .. } => operand,
                };
                super::operand(program, function, operand, out)?;
            }
        }
    }
    write!(out, ")")?;
    if let CallArguments::Object(range) = arguments {
        let objects = function
            .objects
            .as_ref()
            .ok_or_else(|| io::Error::other("missing object call arguments"))?;
        let mut started = false;
        for (index, argument) in objects.arguments[range.start as usize..range.end as usize]
            .iter()
            .enumerate()
        {
            if let BoundaryArgument::Object {
                classes: Some(value),
                ..
            } = argument
            {
                if started {
                    write!(out, ", classes=")?;
                } else {
                    write!(out, " [classes=")?;
                    started = true;
                }
                write!(out, "{index}:")?;
                classes(*value, out)?;
            }
        }
        if started {
            write!(out, "]")?;
        }
    }
    if !matches!(callee, Operand::Symbol(_)) {
        write!(out, " as (")?;
        parameters(program, signature, true, out)?;
        write!(out, ") -> ")?;
        result(program, signature, out)?;
        result_metadata(program, signature, out)?;
        if program.noreturn(signature).map_err(io::Error::other)? {
            write!(out, " [return=noreturn]")?;
        }
    }
    Ok(())
}

pub(super) fn returned(
    program: &Program,
    function: &Function,
    signature: SignatureId,
    value: crate::lowir::program::Terminator,
    out: &mut impl Write,
) -> io::Result<()> {
    use crate::lowir::{program::Terminator, vocabulary::Type as Scalar};
    let expected = match program.signatures.get(signature.0 as usize) {
        Some(Signature::Scalar(data)) => boundary::Type::Scalar(data.result),
        _ => {
            program
                .object_boundary(signature)
                .map_err(io::Error::other)?
                .result
        }
    };
    match (value, expected) {
        (Terminator::Return(Some(value)), boundary::Type::Scalar(ty)) if value.ty == ty => {
            validation::scalar_operand(value.operand).map_err(io::Error::other)?;
            write!(out, "return {} ", ty.name())?;
            super::operand(program, function, value.operand, out)
        }
        (Terminator::Return(None), boundary::Type::Scalar(Scalar::Void)) => {
            write!(out, "return void")
        }
        (Terminator::ReturnObject { operand, span }, boundary::Type::Object(object))
            if function.spans.get(span.0 as usize) == Some(&object.span) =>
        {
            validation::object_operand(
                program,
                function,
                Some(signature),
                boundary::ObjectValue {
                    operand,
                    span: object.span,
                },
            )
            .map_err(io::Error::other)?;
            write!(out, "return ")?;
            ty(expected, out)?;
            write!(out, " ")?;
            super::operand(program, function, operand, out)
        }
        _ => Err(io::Error::other(
            "LowIR return kind or span differs from boundary",
        )),
    }
}
