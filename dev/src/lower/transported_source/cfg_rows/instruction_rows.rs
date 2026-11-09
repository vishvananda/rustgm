//! Version thirteen copies a body's inline assembly payload into its
//! original's cold columns, after the initial profile's preflight. No class,
//! feature or lowering decision is serialized: each consumer re-proves them.
use super::{Body, Context, Declarations, Stage, g, ir, s};
use crate::{
    body::assembly,
    support::initial_assembly::{GENERAL, OUTPUT, XMM},
    syntax::{assembly::Register, ast::List},
    types::{Kind, Primitive, TypeId},
};

/// The profile check runs before any local, projection or value row, so an
/// out-of-profile body's text never depends on the other rows of that body.
pub(super) fn preflight(
    stage: &mut Stage,
    facts: &Declarations,
    body: &Body,
) -> Result<(), &'static str> {
    let Some(data) = body.assembly.as_deref() else {
        return Ok(());
    };
    // The comment scan reads each template byte once.
    stage.charge(
        data.instructions
            .len()
            .saturating_add(data.operands.len())
            .saturating_add(data.pieces.len())
            .saturating_add(text_bytes(data)),
    )?;
    for instruction in &data.instructions {
        assembly::profile::admit(data, instruction, |register, input| {
            let value = body
                .values
                .get(input.0 as usize)
                .ok_or("assembly input lost its value")?;
            input_type(facts, register, value.ty)
        })?;
    }
    Ok(())
}
fn text_bytes(data: &assembly::Payload) -> usize {
    data.templates
        .iter()
        .map(|template| template.text.len())
        .fold(0_usize, usize::saturating_add)
}
/// A `reg` input is a thin raw pointer and an `xmm_reg` input a nominal,
/// which only a 128-bit vector can be once lowering checks its layout.
fn input_type(facts: &Declarations, register: Register, ty: TypeId) -> Result<(), &'static str> {
    match (register, facts.types.kind(ty)) {
        (Register::General, Some(Kind::Pointer { pointee, .. })) => {
            match facts.types.kind(pointee) {
                Some(Kind::Slice(_) | Kind::Dynamic { .. } | Kind::Primitive(Primitive::Str))
                | None => Err(GENERAL),
                Some(_) => Ok(()),
            }
        }
        (Register::General, _) => Err(GENERAL),
        (Register::Xmm, Some(Kind::Adt(..))) => Ok(()),
        (Register::Xmm, _) => Err(XMM),
        (register, _) => Err(assembly::profile::class(register).unwrap_or(GENERAL)),
    }
}
/// The value row of one assembly invocation; its columns are copied once.
pub(super) fn value(kind: ir::Rvalue) -> Result<s::ValueKind, &'static str> {
    let ir::Rvalue::Assembly {
        instruction,
        arguments,
    } = kind
    else {
        return Err("source assembly value changed its actual form");
    };
    Ok(s::ValueKind::Assembly {
        instruction,
        arguments: g::Range {
            start: arguments.start,
            end: arguments.end,
        },
    })
}
/// Copy the payload: sites, operand types and each recorded lexical unsafe
/// context. Templates are copied verbatim, charged per row and per byte.
pub(super) fn emit(
    stage: &mut Stage,
    facts: &mut Declarations,
    (body, context): (&Body, Context),
    raw: &mut s::Original,
    sites: &mut super::super::locations::Sites,
) -> Result<(), &'static str> {
    let Some(data) = body.assembly.as_deref() else {
        return Ok(());
    };
    dense(body, data.instructions.len())?;
    let bytes = text_bytes(data);
    stage.charge(
        data.instructions
            .len()
            .saturating_add(data.templates.len())
            .saturating_add(data.pieces.len())
            .saturating_add(data.operands.len())
            .saturating_add(bytes),
    )?;
    stage.reserve(
        size_of::<s::Assembly>()
            .saturating_add(size_of::<s::AssemblyInstruction>() * data.instructions.len())
            .saturating_add(size_of::<s::AssemblyTemplate>() * data.templates.len())
            .saturating_add(size_of::<s::AssemblyPiece>() * data.pieces.len())
            .saturating_add(size_of::<s::AssemblyOperand>() * data.operands.len())
            .saturating_add(bytes),
    )?;
    let mut rows = s::Assembly {
        instructions: Vec::with_capacity(data.instructions.len()),
        templates: Vec::with_capacity(data.templates.len()),
        pieces: Vec::with_capacity(data.pieces.len()),
        operands: Vec::with_capacity(data.operands.len()),
    };
    for instruction in &data.instructions {
        let node = body.syntax_site(instruction.site)?;
        let safety = facts
            .assembly_contexts
            .as_deref()
            .and_then(|contexts| contexts.get(&node))
            .copied()
            .ok_or("source inline assembly lost its lexical unsafe context")?;
        rows.instructions.push(s::AssemblyInstruction {
            site: sites.body(stage, instruction.site)?,
            templates: range(instruction.templates),
            operands: range(instruction.operands),
            options: instruction.options,
            safety,
        });
    }
    for template in &data.templates {
        rows.templates.push(s::AssemblyTemplate {
            text: template.text.as_str().into(),
            pieces: range(template.pieces),
        });
    }
    rows.pieces
        .extend(data.pieces.iter().map(|piece| s::AssemblyPiece {
            start: piece.start,
            end: piece.end,
            operand: piece.operand,
            modifier: piece.modifier,
        }));
    for operand in &data.operands {
        rows.operands
            .push(operand_row(stage, facts, context, operand)?);
    }
    raw.assembly = Some(Box::new(rows));
    Ok(())
}
fn operand_row(
    stage: &mut Stage,
    facts: &mut Declarations,
    context: Context,
    operand: &assembly::Operand,
) -> Result<s::AssemblyOperand, &'static str> {
    let class = match operand.register {
        Register::General => s::AssemblyClass::General,
        Register::Xmm => s::AssemblyClass::Vector,
        register => return Err(assembly::profile::class(register).unwrap_or(GENERAL)),
    };
    if operand.output.is_some() {
        return Err(OUTPUT);
    }
    let ty = operand.ty.ok_or("source assembly operand lost its type")?;
    stage.prepare_type_arena(facts, ty)?;
    Ok(s::AssemblyOperand {
        class,
        input: operand.input.map(|value| value.0),
        output: false,
        ty: stage.ty(facts, ty, context)?,
        late: operand.late,
    })
}
/// Instructions follow their values densely, as the builder pushes them.
fn dense(body: &Body, count: usize) -> Result<(), &'static str> {
    let mut next = 0_usize;
    for value in &body.values {
        if let ir::Rvalue::Assembly { instruction, .. } = value.kind {
            if instruction as usize != next {
                return Err("source assembly payload leaves its value order");
            }
            next = next.saturating_add(1);
        }
    }
    if next == count {
        Ok(())
    } else {
        Err("source assembly payload leaves its value order")
    }
}
const fn range(list: List) -> g::Range {
    g::Range {
        start: list.start,
        end: list.end,
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/register_rows.rs"]
mod tests;
