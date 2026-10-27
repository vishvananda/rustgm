//! Owned assembly data follows its actual Body through templates and instances.
use super::ir::{Place, ValueId};
use crate::{
    resolve::unit::NodeKey,
    syntax::{assembly::Register, ast::List},
    types::TypeId,
};
#[derive(Clone)]
pub struct Template {
    pub text: String,
    pub pieces: List,
}
#[derive(Clone, Copy)]
pub struct Piece {
    pub start: u32,
    pub end: u32,
    pub operand: u32,
    pub modifier: u8,
}
#[derive(Clone, Copy)]
pub struct Operand {
    pub(crate) register: Register,
    pub input: Option<ValueId>,
    pub output: Option<Place>,
    pub ty: Option<TypeId>,
    pub late: bool,
}
#[derive(Clone)]
pub struct Instruction {
    pub source: NodeKey,
    pub templates: List,
    pub operands: List,
    pub options: u16,
}
#[derive(Clone, Default)]
pub struct Payload {
    pub(crate) source: Option<NodeKey>,
    pub instructions: Vec<Instruction>,
    pub templates: Vec<Template>,
    pub pieces: Vec<Piece>,
    pub operands: Vec<Operand>,
}
impl Payload {
    pub(crate) fn allocation_bytes(&self) -> usize {
        use crate::support::retained::{sum, vector};
        sum([
            vector(&self.instructions),
            vector(&self.templates),
            vector(&self.pieces),
            vector(&self.operands),
            self.templates
                .iter()
                .map(|t| t.text.capacity())
                .fold(0usize, usize::saturating_add),
        ])
    }
    pub(crate) fn growth_bound(
        &self,
        instructions: usize,
        templates: usize,
        pieces: usize,
        operands: usize,
        bytes: usize,
    ) -> usize {
        fn vector<T>(old: &Vec<T>, more: usize) -> usize {
            if old.len().saturating_add(more) <= old.capacity() {
                0
            } else {
                old.capacity()
                    .saturating_add(old.len().saturating_add(more).saturating_mul(2).max(8))
                    .saturating_mul(size_of::<T>())
            }
        }
        vector(&self.instructions, instructions)
            .saturating_add(vector(&self.templates, templates))
            .saturating_add(vector(&self.pieces, pieces))
            .saturating_add(vector(&self.operands, operands))
            .saturating_add(bytes)
    }
}
/// Check a body-local effect against the SAME retained data before proof or emission.
pub(crate) fn instruction(body: &super::ir::Body, id: u32) -> Result<&Instruction, &'static str> {
    let data = body
        .assembly
        .as_deref()
        .ok_or("assembly effect has no retained body data")?;
    if data.source != body.source.source() {
        return Err("assembly payload belongs to another actual source body");
    }
    let instruction = data
        .instructions
        .get(id as usize)
        .ok_or("assembly instruction identity is stale")?;
    if body
        .source
        .source()
        .is_none_or(|source| source.file != instruction.source.file)
    {
        return Err("assembly instruction belongs to another source body");
    }
    let operands = data
        .operands
        .get(instruction.operands.start as usize..instruction.operands.end as usize)
        .ok_or("assembly operand range is invalid")?;
    if data
        .templates
        .get(instruction.templates.start as usize..instruction.templates.end as usize)
        .is_none()
    {
        return Err("assembly template range is invalid");
    }
    let _ = operands;
    Ok(instruction)
}
pub(crate) fn output(
    body: &super::ir::Body,
    effect: ValueId,
    ordinal: u32,
    ty: TypeId,
) -> Result<(), &'static str> {
    let super::ir::Rvalue::Assembly {
        instruction: id,
        arguments,
    } = body
        .values
        .get(effect.0 as usize)
        .ok_or("assembly output lost its effect")?
        .kind
    else {
        return Err("assembly output names another value kind");
    };
    let _ = arguments;
    let row = instruction(body, id)?;
    let operand = body
        .assembly
        .as_deref()
        .and_then(|data| {
            data.operands
                .get(row.operands.start as usize..row.operands.end as usize)
        })
        .and_then(|operands| operands.get(ordinal as usize))
        .ok_or("assembly output operand is stale")?;
    if operand.output.is_none() || operand.ty != Some(ty) {
        return Err("assembly output differs from its retained typed place");
    }
    Ok(())
}

/// Linear cold validation before the ordinary body proof; output lookups stay indexed.
pub(crate) fn validate(body: &super::ir::Body) -> Result<(), &'static str> {
    for value in &body.values {
        match value.kind {
            super::ir::Rvalue::Assembly {
                instruction: id,
                arguments,
            } => {
                let row = instruction(body, id)?;
                let data = body.assembly.as_deref().ok_or("missing assembly data")?;
                let operands =
                    &data.operands[row.operands.start as usize..row.operands.end as usize];
                let inputs = body
                    .arguments
                    .get(arguments.start as usize..arguments.end as usize)
                    .ok_or("assembly argument range is invalid")?;
                if !operands
                    .iter()
                    .filter_map(|o| o.input)
                    .eq(inputs.iter().copied())
                {
                    return Err("assembly input order differs from retained operands");
                }
                for operand in operands {
                    if let Some(id) = operand.input
                        && body
                            .values
                            .get(id.0 as usize)
                            .is_none_or(|v| Some(v.ty) != operand.ty)
                    {
                        return Err("assembly input type differs from retained operand");
                    }
                    if let Some(place) = operand.output
                        && body.place_type(place) != operand.ty
                    {
                        return Err("assembly output type differs from retained place");
                    }
                }
            }
            super::ir::Rvalue::AssemblyOutput { effect, operand } => {
                output(body, effect, operand, value.ty)?;
            }
            _ => {}
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../tests/invariants/asm_effect_contracts.rs"]
mod tests;

/// Prepay actual selected assembly coexistence before a caller clones its body.
/// Ordinary bodies retain their exact allocation path.
pub(crate) fn prepay_instance(
    report: &crate::resolve::expansion::Report,
    body: &super::ir::Body,
) -> Result<(), crate::resolve::package::Error> {
    let Some(data) = body.assembly.as_deref() else {
        return Ok(());
    };
    let first = data.instructions.first().ok_or_else(|| {
        report.body_error(
            body.source,
            "assembly payload contains no actual instruction",
        )
    })?;
    let work = data
        .instructions
        .len()
        .saturating_add(data.templates.len())
        .saturating_add(data.pieces.len())
        .saturating_add(data.operands.len())
        .saturating_add(body.values.len());
    report.prepare_assembly(first.source, work, 0)?;
    let bytes = size_of::<Payload>().saturating_add(data.allocation_bytes());
    report.prepare_assembly(first.source, 1, bytes)?;
    validate(body).map_err(|message| report.body_error(body.source, message))?;
    for instruction in data.instructions.iter().skip(1) {
        report.prepare_assembly(instruction.source, 1, 0)?;
    }
    Ok(())
}
