//! Serialize the typed scalar model using the pinned CPPGM `LowIR` vocabulary.
use super::{
    program::{
        Access, Function, Instruction, Literal, Operand, Operation, Program, SignatureId, Storage,
        Terminator,
    },
    validation,
    vocabulary::Type,
};
use std::io::{self, Write};
mod objects;

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
        temporaries += body.temporaries.len()
            + body
                .objects
                .as_ref()
                .map_or(0, |objects| objects.temporaries.len());
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
        if symbol.body.is_none() {
            write!(out, "declare ")?;
        }
        write!(out, "function @{}(", symbol.name)?;
        parameters(program, symbol.signature, true, out)?;
        write!(out, ") -> ")?;
        objects::result(program, symbol.signature, out)?;
        objects::result_metadata(program, symbol.signature, out)?;
        if symbol.entry {
            write!(out, " [role=entry]")?;
        }
        if program
            .noreturn(symbol.signature)
            .map_err(io::Error::other)?
        {
            write!(out, " [return=noreturn]")?;
        }
        if let Some(function) = &symbol.body {
            writeln!(out, " {{")?;
            function_body(program, symbol.signature, function, out)?;
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
    objects::parameters(program, signature, names, out)
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
fn function_body(
    program: &Program,
    signature: SignatureId,
    function: &Function,
    out: &mut impl Write,
) -> io::Result<()> {
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
            validation::scalar_operation(function, inst).map_err(io::Error::other)?;
            if let Some(temp) = inst.result {
                match temp {
                    super::program::ResultId::Scalar(id) => write!(out, "%t{} = ", id.0)?,
                    super::program::ResultId::Object(id) => write!(out, "%o{} = ", id.0)?,
                }
            }
            instruction(program, signature, function, inst, out)?;
            writeln!(out)?;
        }
        write!(out, "    ")?;
        match block.terminator {
            Terminator::Jump(target) => write!(out, "jump ^b{}", target.0)?,
            Terminator::Branch { condition, yes, no } => {
                validation::scalar_operand(condition).map_err(io::Error::other)?;
                write!(out, "branch ")?;
                operand(program, function, condition, out)?;
                write!(out, ", ^b{}, ^b{}", yes.0, no.0)?;
            }
            value @ (Terminator::Return(_) | Terminator::ReturnObject { .. }) => {
                objects::returned(program, function, signature, value, out)?;
            }
            Terminator::Unreachable => write!(out, "unreachable")?,
        }
        writeln!(out)?;
    }
    Ok(())
}
fn result_type(function: &Function, inst: &Instruction) -> io::Result<Type> {
    let Some(super::program::ResultId::Scalar(temp)) = inst.result else {
        return Err(io::Error::other(
            "LowIR scalar instruction lacks scalar result",
        ));
    };
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
    signature: SignatureId,
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
            validation::copy_source(program, function, signature, source, *span)
                .map_err(io::Error::other)?;
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
        Operation::Call { .. } => return objects::call(program, function, signature, inst, out),
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
        Operand::Temporary(id) => {
            if function.temporaries.get(id.0 as usize).is_none() {
                return Err(io::Error::other("invalid scalar temporary"));
            }
            write!(out, "%t{}", id.0)
        }
        Operand::ObjectTemporary(id) => {
            if function
                .objects
                .as_ref()
                .and_then(|objects| objects.temporaries.get(id.0 as usize))
                .and_then(|span| function.spans.get(span.0 as usize))
                .is_none()
            {
                return Err(io::Error::other("invalid object temporary"));
            }
            write!(out, "%o{}", id.0)
        }
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
