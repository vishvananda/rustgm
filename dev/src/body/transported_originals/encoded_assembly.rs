//! Version-thirteen inline assembly is rebuilt into the consumer's own body
//! payload. The consumer's validation checked its columns; every class,
//! CPU feature and lowering decision is re-proved here or at lowering.
use super::ImportedOriginals;
use crate::{
    body::{
        assembly::{Instruction, Operand, Payload, Piece, Template},
        ir::{Body, Rvalue, ValueId, ValueList},
    },
    sema::{
        bodies::assembly_values::simd_operand, crates::SourceOriginal, model::Declarations,
        target_features::FeatureSet,
    },
    support::{
        crates::generics::source as s,
        initial_assembly::{ABCD, BYTE, CONSTANT, EXPLICIT},
    },
    syntax::{assembly::Register, ast::List},
};

pub(super) fn value(view: &SourceOriginal<'_>, kind: s::ValueKind) -> Result<Rvalue, &'static str> {
    let s::ValueKind::Assembly {
        instruction,
        arguments,
    } = kind
    else {
        return Err("source assembly reconstruction belongs to another operation");
    };
    if !view.rows().descriptor().profile().body_nodes() {
        return Err("source inline assembly requires the version-thirteen body IR profile");
    }
    Ok(Rvalue::Assembly {
        instruction,
        arguments: ValueList {
            start: arguments.start,
            end: arguments.end,
        },
    })
}
/// Whether a cached assembly value is the one its row rebuilds; `None` for
/// every other pair of kinds.
pub(super) const fn same(actual: Rvalue, record: s::ValueKind) -> Option<bool> {
    match (actual, record) {
        (
            Rvalue::Assembly {
                instruction,
                arguments,
            },
            s::ValueKind::Assembly {
                instruction: ordinal,
                arguments: raw,
            },
        ) => {
            Some(instruction == ordinal && arguments.start == raw.start && arguments.end == raw.end)
        }
        (Rvalue::Assembly { .. }, _) | (_, s::ValueKind::Assembly { .. }) => Some(false),
        _ => None,
    }
}
/// The body's payload, owned by the body's own origin; one step per row and
/// per template byte.
pub(super) fn payload(
    view: &SourceOriginal<'_>,
    body: &mut Body,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    let Some(rows) = view.raw().assembly.as_deref() else {
        return Ok(());
    };
    if body.const_context {
        return Err(CONSTANT);
    }
    work.charge(
        rows.instructions
            .len()
            .saturating_add(rows.templates.len())
            .saturating_add(rows.pieces.len())
            .saturating_add(rows.operands.len()),
    )?;
    let mut data = Payload {
        source: Some(body.source),
        instructions: Vec::with_capacity(rows.instructions.len()),
        templates: Vec::with_capacity(rows.templates.len()),
        pieces: rows.pieces.iter().map(piece).collect(),
        operands: Vec::with_capacity(rows.operands.len()),
    };
    for row in &rows.instructions {
        data.instructions.push(Instruction {
            site: view.site(row.site, &mut work.steps)?,
            templates: list(row.templates),
            operands: list(row.operands),
            options: row.options,
        });
    }
    for row in &rows.templates {
        work.charge(row.text.len())?;
        data.templates.push(Template {
            text: String::from(&*row.text),
            pieces: list(row.pieces),
        });
    }
    for row in &rows.operands {
        data.operands.push(operand(view, row, &mut work.steps)?);
    }
    body.assembly = Some(Box::new(data));
    Ok(())
}
fn operand(
    view: &SourceOriginal<'_>,
    row: &s::AssemblyOperand,
    steps: &mut usize,
) -> Result<Operand, &'static str> {
    Ok(Operand {
        register: match row.class {
            s::AssemblyClass::General => Register::General,
            s::AssemblyClass::Vector => Register::Xmm,
            s::AssemblyClass::Abcd => return Err(ABCD),
            s::AssemblyClass::Byte => return Err(BYTE),
            s::AssemblyClass::Explicit => return Err(EXPLICIT),
        },
        input: row.input.map(ValueId),
        output: None,
        ty: Some(view.ty(row.ty, steps)?),
        late: row.late,
    })
}
const fn piece(row: &s::AssemblyPiece) -> Piece {
    Piece {
        start: row.start,
        end: row.end,
        operand: row.operand,
        modifier: row.modifier,
    }
}
const fn list(range: crate::support::crates::generics::Range) -> List {
    List {
        start: range.start,
        end: range.end,
    }
}
/// The retained bytes of a rebuilt payload.
pub(super) fn allocation(record: &s::Original) -> usize {
    record.assembly.as_deref().map_or(0, |rows| {
        size_of::<Payload>()
            .saturating_add(size_of::<Instruction>() * rows.instructions.len())
            .saturating_add(size_of::<Template>() * rows.templates.len())
            .saturating_add(size_of::<Piece>() * rows.pieces.len())
            .saturating_add(size_of::<Operand>() * rows.operands.len())
            .saturating_add(
                rows.templates
                    .iter()
                    .map(|row| row.text.len())
                    .fold(0, usize::saturating_add),
            )
    })
}
/// A cached body keeps exactly the payload its original's columns rebuild.
pub(super) fn validate(
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<(), &'static str> {
    const CHANGED: &str = "source original changed its actual assembly payload";
    let (rows, data) = match (view.raw().assembly.as_deref(), body.assembly.as_deref()) {
        (None, None) => return Ok(()),
        (Some(rows), Some(data)) => (rows, data),
        _ => return Err(CHANGED),
    };
    if data.source != Some(body.source)
        || data.instructions.len() != rows.instructions.len()
        || data.templates.len() != rows.templates.len()
        || data.pieces.len() != rows.pieces.len()
        || data.operands.len() != rows.operands.len()
    {
        return Err(CHANGED);
    }
    for (actual, row) in data.instructions.iter().zip(&rows.instructions) {
        if actual.site != view.site(row.site, steps)?
            || actual.templates != list(row.templates)
            || actual.operands != list(row.operands)
            || actual.options != row.options
        {
            return Err(CHANGED);
        }
    }
    for (actual, row) in data.templates.iter().zip(&rows.templates) {
        *steps = steps.saturating_add(1).saturating_add(row.text.len());
        if actual.text != *row.text || actual.pieces != list(row.pieces) {
            return Err(CHANGED);
        }
    }
    for (actual, row) in data.pieces.iter().zip(&rows.pieces) {
        let expected = piece(row);
        if (actual.start, actual.end, actual.operand, actual.modifier)
            != (
                expected.start,
                expected.end,
                expected.operand,
                expected.modifier,
            )
        {
            return Err(CHANGED);
        }
    }
    for (actual, row) in data.operands.iter().zip(&rows.operands) {
        let expected = operand(view, row, steps)?;
        if actual.register != expected.register
            || actual.input != expected.input
            || actual.output.is_some()
            || actual.ty != expected.ty
            || actual.late != expected.late
        {
            return Err(CHANGED);
        }
    }
    *steps = steps.saturating_add(rows.instructions.len().saturating_add(rows.operands.len()));
    Ok(())
}
/// Each `xmm_reg` input is re-proved against the consumer's own facts: a
/// 128-bit vector, under the consumer's target baseline plus the closure of
/// the owning function's imported declared target features, if any.
pub(super) fn check(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    work: &mut ImportedOriginals,
) -> Result<(), &'static str> {
    let Some(data) = body.assembly.as_deref() else {
        return Ok(());
    };
    work.charge(data.operands.len())?;
    let mut available = None;
    for operand in &data.operands {
        if operand.register != Register::Xmm {
            continue;
        }
        simd_operand(
            facts,
            operand.ty.ok_or("source assembly operand lost its type")?,
        )?;
        let (baseline, imported) = match available {
            Some(features) => features,
            None => *available.insert((
                FeatureSet::target_baseline(|growth| work.charge(growth.work), |message| message)?,
                crate::sema::target_features::imported(facts, view.definition())?
                    .unwrap_or_default(),
            )),
        };
        baseline.admit_imported_xmm(imported)?;
    }
    Ok(())
}
