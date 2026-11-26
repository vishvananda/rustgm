//! Inline assembly columns: v13 admits the initial profile; v14 additionally
//! carries integer outputs, exact fixed GPR aliases and full-register views.
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
    if read.profile.declaration_policies() {
        profile::flags_option(options)?;
    } else {
        profile::option(options)?;
    }
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
    let modifier = read.reader.u8()?;
    if modifier != 0 && !(matches!(modifier, b'e' | b'r') && read.profile.declaration_policies()) {
        return Err(profile::MODIFIER);
    }
    Ok(s::AssemblyPiece {
        start,
        end,
        operand,
        modifier,
    })
}
fn operand_write(out: &mut Writer, row: &s::AssemblyOperand) -> Result<(), &'static str> {
    // Tag four has an exact fixed-register payload only in version fourteen.
    let tag = match row.class {
        s::AssemblyClass::General => 0,
        s::AssemblyClass::Abcd => return Err(profile::ABCD),
        s::AssemblyClass::Byte => return Err(profile::BYTE),
        s::AssemblyClass::Vector => 3,
        s::AssemblyClass::Explicit { .. } => 4,
    };
    out.u8(tag)?;
    if let s::AssemblyClass::Explicit { base, bytes, xmm } = row.class {
        out.u8(base)?;
        out.u8(bytes)?;
        out.u8(u8::from(xmm))?;
    }
    option(out, row.input)?;
    out.u8(u8::from(row.output.is_some()))?;
    if let Some(place) = row.output {
        super::ir_nodes::place_write(out, place)?;
    }
    id(out, row.ty.0)?;
    out.u8(u8::from(row.late))
}
fn operand_read(read: &mut Input<'_, '_>) -> Result<s::AssemblyOperand, &'static str> {
    let class = match read.reader.u8()? {
        0 => s::AssemblyClass::General,
        1 => return Err(profile::ABCD),
        2 => return Err(profile::BYTE),
        3 => s::AssemblyClass::Vector,
        4 if read.profile.declaration_policies() => s::AssemblyClass::Explicit {
            base: read.reader.u8()?,
            bytes: read.reader.u8()?,
            xmm: read.boolean()?,
        },
        4 => return Err(profile::EXPLICIT),
        _ => return Err("unknown source assembly register class"),
    };
    let input = read.option()?;
    let output = match read.reader.u8()? {
        0 => None,
        1 if read.profile.declaration_policies() => Some(super::ir_nodes::place_read(read)?),
        _ => return Err(profile::OUTPUT),
    };
    Ok(s::AssemblyOperand {
        class,
        input,
        output,
        ty: super::g::TypeRef(read.id()?),
        late: read.boolean()?,
    })
}
