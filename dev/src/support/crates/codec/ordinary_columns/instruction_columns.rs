//! Version thirteen's cold inline assembly columns. Only the initial assembly
//! profile's forms have a wire layout: every other defined tag, bit or byte
//! is refused by the profile's own text, and no reserved payload is written
//! or read.
use super::{Input, Writer, crates, id, option, range, s, write};
use crate::support::initial_assembly as profile;

pub(super) fn assembly_write(out: &mut Writer, rows: &s::Assembly) -> Result<(), &'static str> {
    write(out, &rows.instructions, instruction_write)?;
    write(out, &rows.templates, |out, row| {
        out.string(&row.text)?;
        range(out, row.pieces)
    })?;
    write(out, &rows.pieces, |out, row| {
        for value in [row.start, row.end, row.operand] {
            id(out, value)?;
        }
        out.u8(row.modifier)
    })?;
    write(out, &rows.operands, operand_write)
}
pub(super) fn assembly_read(read: &mut Input<'_, '_>) -> Result<s::Assembly, &'static str> {
    Ok(s::Assembly {
        instructions: read.list(crates::MAX_COMPONENTS, 23, instruction_read)?,
        templates: read.list(crates::MAX_COMPONENTS, 12, |read| {
            Ok(s::AssemblyTemplate {
                text: super::operation_columns::text_read(read)?,
                pieces: read.range()?,
            })
        })?,
        pieces: read.list(crates::MAX_COMPONENTS, 13, piece_read)?,
        operands: read.list(crates::MAX_COMPONENTS, 8, operand_read)?,
    })
}
fn instruction_write(out: &mut Writer, row: &s::AssemblyInstruction) -> Result<(), &'static str> {
    id(out, row.site)?;
    range(out, row.templates)?;
    range(out, row.operands)?;
    out.u16(row.options)?;
    out.u8(match row.safety {
        s::CallSafety::Safe => 0,
        s::CallSafety::ExplicitUnsafeBlock => 1,
        s::CallSafety::UnsafeFunction => 2,
    })
}
fn instruction_read(read: &mut Input<'_, '_>) -> Result<s::AssemblyInstruction, &'static str> {
    let site = read.id()?;
    let templates = read.range()?;
    let operands = read.range()?;
    let options = read.reader.u16()?;
    if options & !profile::KNOWN != 0 {
        return Err("unknown source assembly option");
    }
    profile::option(options)?;
    let safety = match read.reader.u8()? {
        0 => return Err("source inline assembly has no unsafe context"),
        1 => s::CallSafety::ExplicitUnsafeBlock,
        2 => s::CallSafety::UnsafeFunction,
        _ => return Err("unknown source inline assembly safety"),
    };
    Ok(s::AssemblyInstruction {
        site,
        templates,
        operands,
        options,
        safety,
    })
}
fn piece_read(read: &mut Input<'_, '_>) -> Result<s::AssemblyPiece, &'static str> {
    let (start, end, operand) = (read.id()?, read.id()?, read.id()?);
    if read.reader.u8()? != 0 {
        return Err(profile::MODIFIER);
    }
    Ok(s::AssemblyPiece {
        start,
        end,
        operand,
        modifier: 0,
    })
}
fn operand_write(out: &mut Writer, row: &s::AssemblyOperand) -> Result<(), &'static str> {
    // Reserved class tags have no payload layout, so neither side has one.
    let tag = match row.class {
        s::AssemblyClass::General => 0,
        s::AssemblyClass::Abcd => return Err(profile::ABCD),
        s::AssemblyClass::Byte => return Err(profile::BYTE),
        s::AssemblyClass::Vector => 3,
        s::AssemblyClass::Explicit => return Err(profile::EXPLICIT),
    };
    out.u8(tag)?;
    option(out, row.input)?;
    out.u8(u8::from(row.output))?;
    id(out, row.ty.0)?;
    out.u8(u8::from(row.late))
}
fn operand_read(read: &mut Input<'_, '_>) -> Result<s::AssemblyOperand, &'static str> {
    let class = match read.reader.u8()? {
        0 => s::AssemblyClass::General,
        1 => return Err(profile::ABCD),
        2 => return Err(profile::BYTE),
        3 => s::AssemblyClass::Vector,
        4 => return Err(profile::EXPLICIT),
        _ => return Err("unknown source assembly register class"),
    };
    let input = read.option()?;
    if read.reader.u8()? != 0 {
        return Err(profile::OUTPUT);
    }
    Ok(s::AssemblyOperand {
        class,
        input,
        output: false,
        ty: super::g::TypeRef(read.id()?),
        late: read.boolean()?,
    })
}
