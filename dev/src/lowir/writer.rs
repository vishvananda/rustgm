//! Serialize the typed scalar model using the pinned CPPGM `LowIR` vocabulary.
use super::{
    program::{
        Access, Function, Instruction, Literal, Operand, Operation, Program, SignatureId, Storage,
        Terminator,
    },
    vocabulary::Type,
};
use std::io::{self, Write};

/// Count owned, already constructed records without rendering them.
///
/// # Errors
/// Propagates output errors.
pub fn count(program: &Program, out: &mut impl Write) -> io::Result<()> {
    let (mut functions, mut blocks, mut instructions, mut slots, mut temporaries) =
        (0_usize, 0, 0, 0, 0);
    for body in program
        .symbols
        .iter()
        .filter_map(|symbol| symbol.body.as_ref())
    {
        functions += 1;
        blocks += body.blocks.len();
        instructions += body.instructions.len();
        slots += body.slots.len();
        temporaries += body.temporaries.len();
    }
    write!(
        out,
        "symbols {} functions {functions} blocks {blocks} instructions {instructions} slots {slots} temporaries {temporaries}",
        program.symbols.len() + program.globals.len()
    )?;
    if !program.globals.is_empty() {
        write!(
            out,
            " globals {} data_items {}",
            program.globals.len(),
            program.data.len()
        )?;
    }
    writeln!(out)
}

/// # Errors
/// Propagates output errors and inconsistent typed identities.
pub fn write(program: &Program, out: &mut impl Write) -> io::Result<()> {
    globals(program, out)?;
    for symbol in &program.symbols {
        let signature = program
            .signatures
            .get(symbol.signature.0 as usize)
            .ok_or_else(|| io::Error::other("invalid LowIR signature"))?;
        if symbol.body.is_none() {
            write!(out, "declare ")?;
        }
        write!(out, "function @{}(", symbol.name)?;
        parameters(program, symbol.signature, true, out)?;
        write!(out, ") -> {}", signature.result.name())?;
        if symbol.entry {
            write!(out, " [role=entry]")?;
        }
        if signature.noreturn {
            write!(out, " [return=noreturn]")?;
        }
        if let Some(function) = &symbol.body {
            writeln!(out, " {{")?;
            function_body(program, function, out)?;
            writeln!(out, "}}")?;
        } else {
            writeln!(out)?;
        }
    }
    Ok(())
}
fn parameters(
    program: &Program,
    signature: SignatureId,
    names: bool,
    out: &mut impl Write,
) -> io::Result<()> {
    for (index, ty) in program.parameters(signature).iter().enumerate() {
        if index != 0 {
            write!(out, ", ")?;
        }
        if names {
            write!(out, "%p{index} : ")?;
        }
        write!(out, "{}", ty.name())?;
    }
    Ok(())
}
fn globals(program: &Program, out: &mut impl Write) -> io::Result<()> {
    use super::data::{GlobalKind, Item};
    for global in &program.globals {
        let items = if global.kind == GlobalKind::Declaration {
            write!(out, "declare ")?;
            None
        } else {
            let range = global
                .items
                .ok_or_else(|| io::Error::other("uninitialized LowIR global"))?;
            Some(
                program
                    .data
                    .get(range.start as usize..range.end as usize)
                    .ok_or_else(|| io::Error::other("invalid LowIR global item range"))?,
            )
        };
        write!(
            out,
            "global @{} : obj<{}x{}>",
            global.name, global.span.bytes, global.span.align
        )?;
        if global.readonly {
            write!(out, " [storage=readonly]")?;
        }
        let Some(items) = items else {
            writeln!(out)?;
            continue;
        };
        writeln!(out, " = {{")?;
        for &item in items {
            match item {
                Item::Zero(bytes) => writeln!(out, "  zero {bytes}")?,
                Item::Bits { ty, bits } => writeln!(out, "  {} {bits}", ty.name())?,
                Item::Address { target, addend } => {
                    write!(
                        out,
                        "  ptr addr @{}",
                        program.data_target(target).map_err(io::Error::other)?
                    )?;
                    if addend > 0 {
                        write!(out, " + {addend}")?;
                    } else if addend < 0 {
                        write!(out, " - {}", addend.unsigned_abs())?;
                    }
                    writeln!(out)?;
                }
            }
        }
        writeln!(out, "}}")?;
    }
    Ok(())
}
fn function_body(program: &Program, function: &Function, out: &mut impl Write) -> io::Result<()> {
    for (index, storage) in function.slots.iter().enumerate() {
        write!(out, "  slot $s{index} : ")?;
        match storage {
            Storage::Scalar(ty) => write!(out, "{}", ty.name())?,
            Storage::Object(id) => {
                let span = function
                    .spans
                    .get(id.0 as usize)
                    .ok_or_else(|| io::Error::other("invalid LowIR object span"))?;
                write!(out, "obj<{}x{}>", span.bytes, span.align)?;
            }
        }
        writeln!(out)?;
    }
    for id in &function.order {
        let block = function
            .blocks
            .get(id.0 as usize)
            .ok_or_else(|| io::Error::other("invalid LowIR block"))?;
        writeln!(out, "  block ^b{}:", id.0)?;
        for inst in &function.instructions
            [block.instructions.start as usize..block.instructions.end as usize]
        {
            write!(out, "    ")?;
            if let Some(temp) = inst.result {
                write!(out, "%t{} = ", temp.0)?;
            }
            instruction(program, function, inst, out)?;
            writeln!(out)?;
        }
        write!(out, "    ")?;
        match block.terminator {
            Terminator::Jump(target) => write!(out, "jump ^b{}", target.0)?,
            Terminator::Branch { condition, yes, no } => {
                write!(out, "branch ")?;
                operand(program, function, condition, out)?;
                write!(out, ", ^b{}, ^b{}", yes.0, no.0)?;
            }
            Terminator::Return(Some(value)) => {
                write!(out, "return {} ", value.ty.name())?;
                operand(program, function, value.operand, out)?;
            }
            Terminator::Return(None) => write!(out, "return void")?,
            Terminator::Unreachable => write!(out, "unreachable")?,
        }
        writeln!(out)?;
    }
    Ok(())
}
fn result_type(function: &Function, inst: &Instruction) -> io::Result<Type> {
    let temp = inst
        .result
        .ok_or_else(|| io::Error::other("LowIR instruction lacks result"))?;
    function
        .temporaries
        .get(temp.0 as usize)
        .copied()
        .ok_or_else(|| io::Error::other("invalid LowIR temporary"))
}
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive dispatch spells typed LowIR instructions and their shared operand tail"
)]
fn instruction(
    program: &Program,
    function: &Function,
    inst: &Instruction,
    out: &mut impl Write,
) -> io::Result<()> {
    let value = match inst.operation {
        Operation::Copy(value) => {
            write!(out, "copy {} ", result_type(function, inst)?.name())?;
            value.operand
        }
        Operation::Address(value) => {
            write!(out, "addr ")?;
            value
        }
        Operation::Index {
            element,
            base,
            offset,
        } => {
            write!(out, "index {} ", element.name())?;
            operand(program, function, base, out)?;
            write!(out, ", ")?;
            offset
        }
        Operation::CopyObject {
            span,
            source,
            destination,
        } => {
            let span = function
                .spans
                .get(span.0 as usize)
                .ok_or_else(|| io::Error::other("invalid LowIR copy span"))?;
            write!(out, "copyobj {}x{} ", span.bytes, span.align)?;
            operand(program, function, source, out)?;
            write!(out, ", ")?;
            destination
        }
        Operation::AtomicCompareExchange {
            value,
            address,
            expected,
            ordering,
        } => {
            write!(out, "atomic_compare_exchange {} ", value.ty.name())?;
            operand(program, function, address, out)?;
            write!(out, ", ")?;
            operand(program, function, expected, out)?;
            write!(out, ", ")?;
            operand(program, function, value.operand, out)?;
            return write!(
                out,
                ", {}, {}",
                ordering.success_encoding(),
                ordering.failure_encoding()
            );
        }
        Operation::AtomicExchange {
            value,
            address,
            ordering,
        }
        | Operation::AtomicAddFetch {
            value,
            address,
            ordering,
        } => {
            let name = match inst.operation {
                Operation::AtomicExchange { .. } => "atomic_exchange",
                _ => "atomic_add_fetch",
            };
            write!(out, "{name} {} ", value.ty.name())?;
            operand(program, function, address, out)?;
            write!(out, ", ")?;
            operand(program, function, value.operand, out)?;
            return write!(out, ", {}", ordering.encoding());
        }
        Operation::AtomicLoad {
            ty,
            address,
            ordering,
        } => {
            write!(out, "atomic_load {} ", ty.name())?;
            operand(program, function, address, out)?;
            return write!(out, ", {}", ordering.encoding());
        }
        Operation::AtomicStore {
            value,
            address,
            ordering,
        } => {
            write!(out, "atomic_store {} ", value.ty.name())?;
            operand(program, function, value.operand, out)?;
            write!(out, ", ")?;
            operand(program, function, address, out)?;
            return write!(out, ", {}", ordering.encoding());
        }
        Operation::Load {
            access,
            ty,
            address,
        } => {
            write!(out, "load{} {} ", access_modifier(access), ty.name())?;
            address
        }
        Operation::Store {
            access,
            value,
            address,
        } => {
            write!(out, "store{} {} ", access_modifier(access), value.ty.name())?;
            operand(program, function, value.operand, out)?;
            write!(out, ", ")?;
            address
        }
        Operation::Unary { op, value } => {
            write!(out, "unary {} {} ", op.name(), value.ty.name())?;
            value.operand
        }
        Operation::Binary { op, left, right } => {
            write!(out, "binary {} {} ", op.name(), left.ty.name())?;
            operand(program, function, left.operand, out)?;
            write!(out, ", ")?;
            right
        }
        Operation::Compare {
            predicate,
            left,
            right,
        } => {
            write!(out, "cmp {} {} ", predicate.name(), left.ty.name())?;
            operand(program, function, left.operand, out)?;
            write!(out, ", ")?;
            right
        }
        Operation::Convert { op, value } => {
            write!(
                out,
                "convert {} {} {} ",
                op.name(),
                result_type(function, inst)?.name(),
                value.ty.name()
            )?;
            value.operand
        }
        Operation::Call {
            signature,
            callee,
            arguments,
        } => {
            let data = program
                .signatures
                .get(signature.0 as usize)
                .ok_or_else(|| io::Error::other("invalid call signature"))?;
            write!(out, "call {} ", data.result.name())?;
            operand(program, function, callee, out)?;
            write!(out, "(")?;
            for (index, value) in function.arguments
                [arguments.start as usize..arguments.end as usize]
                .iter()
                .enumerate()
            {
                if index != 0 {
                    write!(out, ", ")?;
                }
                operand(program, function, value.operand, out)?;
            }
            write!(out, ")")?;
            if !matches!(callee, Operand::Symbol(_)) {
                write!(out, " as (")?;
                parameters(program, signature, true, out)?;
                write!(out, ") -> {}", data.result.name())?;
                if data.noreturn {
                    write!(out, " [return=noreturn]")?;
                }
            }
            return Ok(());
        }
    };
    operand(program, function, value, out)
}
fn operand(
    program: &Program,
    function: &Function,
    value: Operand,
    out: &mut impl Write,
) -> io::Result<()> {
    match value {
        Operand::Temporary(id) => write!(out, "%t{}", id.0),
        Operand::Parameter(index) => write!(out, "%p{index}"),
        Operand::Slot(id) => write!(out, "$s{}", id.0),
        Operand::Symbol(id) => write!(
            out,
            "@{}",
            program
                .symbols
                .get(id.0 as usize)
                .ok_or_else(|| io::Error::other("invalid LowIR symbol"))?
                .name
        ),
        Operand::Global(id) => write!(
            out,
            "@{}",
            program
                .data_target(super::data::Target::Global(id))
                .map_err(io::Error::other)?
        ),
        Operand::Literal(id) => {
            let literal = function
                .literals
                .get(id.0 as usize)
                .ok_or_else(|| io::Error::other("invalid LowIR literal"))?;
            match *literal {
                Literal::Integer(bits) => write!(out, "{bits}"),
                Literal::Null => write!(out, "nullptr"),
                Literal::F32(bits) => floating(f64::from(f32::from_bits(bits)), out),
                Literal::F64(bits) => floating(f64::from_bits(bits), out),
            }
        }
    }
}
fn access_modifier(access: Access) -> &'static str {
    match access {
        Access::Ordinary => "",
        Access::Volatile => " volatile",
    }
}
fn floating(value: f64, out: &mut impl Write) -> io::Result<()> {
    if value.is_nan() {
        write!(out, "nan")
    } else if value == f64::INFINITY {
        write!(out, "inf")
    } else if value == f64::NEG_INFINITY {
        write!(out, "-inf")
    } else {
        write!(out, "{value:.17e}")
    }
}
