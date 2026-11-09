//! Version thirteen's assembly columns are exact before any proof: dense
//! ordinals, partitioned ranges, inputs equal to the value's arguments,
//! well-formed placeholders and the initial assembly profile, whose comment
//! check comes last. The consumer still re-proves Copy inputs, CPU features
//! and lowering over its own body.
use super::{Context, DomainIndex, Work, g, s};
use crate::support::initial_assembly as profile;

impl DomainIndex<'_, '_> {
    /// `unsafe_` is the defining header's own unsafety, the only context a
    /// recorded `UnsafeFunction` may name.
    pub(in super::super) fn assembly(
        &mut self,
        context: Context,
        original: &s::Original,
        unsafe_: bool,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let count = ordinals(original, work)?;
        let Some(rows) = original.assembly.as_deref() else {
            return if count == 0 {
                Ok(())
            } else {
                Err("source assembly column differs from its assembly values")
            };
        };
        if count == 0 || count != rows.instructions.len() {
            return Err("source assembly column differs from its assembly values");
        }
        if original.context != s::CallContext::Runtime {
            return Err(profile::CONSTANT);
        }
        if !partition(
            rows.instructions.iter().map(|row| row.templates),
            rows.templates.len(),
            work,
        )? {
            return Err("source assembly template ranges do not partition their column");
        }
        if !partition(
            rows.instructions.iter().map(|row| row.operands),
            rows.operands.len(),
            work,
        )? {
            return Err("source assembly operand ranges do not partition their column");
        }
        if !partition(
            rows.templates.iter().map(|row| row.pieces),
            rows.pieces.len(),
            work,
        )? {
            return Err("source assembly piece ranges do not partition their column");
        }
        for instruction in &rows.instructions {
            Self::site(original, instruction.site, work)?;
            instruction_profile(instruction, unsafe_)?;
            for operand in instruction.operands.slice(&rows.operands)? {
                self.types.root(self.headers, context, operand.ty, work)?;
            }
            placeholders(rows, instruction, work)?;
        }
        for value in &original.values {
            if let s::ValueKind::Assembly {
                instruction,
                arguments,
            } = value.kind
            {
                if self.headers.rows().types.get(value.ty.0 as usize)
                    != Some(&s::Type::Scalar(crate::support::crates::Scalar::Unit))
                {
                    return Err("source assembly value is not a unit effect");
                }
                let row = &rows.instructions[instruction as usize];
                self.inputs(
                    original,
                    row.operands.slice(&rows.operands)?,
                    arguments,
                    work,
                )?;
            }
        }
        for instruction in &rows.instructions {
            comments(rows, instruction, work)?;
        }
        Ok(())
    }
    /// Each operand is an input whose value is the next argument and whose
    /// type is that value's, in the profile's class.
    fn inputs(
        &self,
        original: &s::Original,
        operands: &[s::AssemblyOperand],
        arguments: g::Range,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let arguments = arguments.slice(&original.arguments)?;
        if operands.len() != arguments.len() {
            return Err("assembly input order differs from retained operands");
        }
        for (operand, &argument) in operands.iter().zip(arguments) {
            work.charge(1)?;
            if operand.output {
                return Err(profile::OUTPUT);
            }
            if operand.input != Some(argument) {
                return Err("assembly input order differs from retained operands");
            }
            if original.values[argument as usize].ty != operand.ty {
                return Err("assembly input type differs from retained operand");
            }
            if operand.late {
                return Err("source assembly input cannot be late");
            }
            self.class(operand)?;
        }
        Ok(())
    }
    /// A `reg` input is a raw pointer to a sized pointee; an `xmm_reg` input
    /// is a nominal the consumer's lowering proves a 128-bit vector.
    fn class(&self, operand: &s::AssemblyOperand) -> Result<(), &'static str> {
        let types = &self.headers.rows().types;
        let ty = |reference: g::TypeRef| types.get(reference.0 as usize);
        match operand.class {
            s::AssemblyClass::General => match ty(operand.ty) {
                Some(s::Type::RawPointer { pointee, .. })
                    if !matches!(ty(*pointee), Some(s::Type::Slice(_) | s::Type::Str) | None) =>
                {
                    Ok(())
                }
                _ => Err(profile::GENERAL),
            },
            s::AssemblyClass::Vector => match ty(operand.ty) {
                Some(s::Type::Nominal { .. }) => Ok(()),
                _ => Err(profile::XMM),
            },
            s::AssemblyClass::Abcd => Err(profile::ABCD),
            s::AssemblyClass::Byte => Err(profile::BYTE),
            s::AssemblyClass::Explicit => Err(profile::EXPLICIT),
        }
    }
}
/// Assembly ordinals follow value order densely; returns their count.
fn ordinals(original: &s::Original, work: &mut Work) -> Result<usize, &'static str> {
    let mut next = 0_u32;
    for value in &original.values {
        work.charge(1)?;
        if let s::ValueKind::Assembly { instruction, .. } = value.kind {
            if instruction != next {
                return Err("source assembly ordinal leaves its value order");
            }
            next = next.saturating_add(1);
        }
    }
    Ok(next as usize)
}
/// Whether the ranges are contiguous, in order, from zero to the column's end.
fn partition(
    ranges: impl Iterator<Item = g::Range>,
    length: usize,
    work: &mut Work,
) -> Result<bool, &'static str> {
    let mut next = 0_u32;
    for range in ranges {
        work.charge(1)?;
        if range.start != next || range.end < range.start {
            return Ok(false);
        }
        next = range.end;
    }
    Ok(next as usize == length)
}
/// The instruction's own options and safety, in the profile's order.
fn instruction_profile(row: &s::AssemblyInstruction, unsafe_: bool) -> Result<(), &'static str> {
    if row.options & !profile::KNOWN != 0 {
        return Err("unknown source assembly option");
    }
    profile::option(row.options)?;
    match row.safety {
        s::CallSafety::Safe => Err("source inline assembly has no unsafe context"),
        s::CallSafety::ExplicitUnsafeBlock => Ok(()),
        s::CallSafety::UnsafeFunction if unsafe_ => Ok(()),
        s::CallSafety::UnsafeFunction => {
            Err("source inline assembly has no matching unsafe context")
        }
    }
}
/// Pieces are sorted, disjoint `{...}` spans on character boundaries of their
/// template, each naming an operand of its instruction, with no modifier.
fn placeholders(
    rows: &s::Assembly,
    instruction: &s::AssemblyInstruction,
    work: &mut Work,
) -> Result<(), &'static str> {
    const MALFORMED: &str = "source assembly placeholder is malformed";
    let operands = instruction
        .operands
        .end
        .saturating_sub(instruction.operands.start);
    for template in instruction.templates.slice(&rows.templates)? {
        let mut next = 0_usize;
        for piece in template.pieces.slice(&rows.pieces)? {
            work.charge(1)?;
            let (start, end) = (piece.start as usize, piece.end as usize);
            let text = template.text.get(start..end).ok_or(MALFORMED)?;
            if start < next || !text.starts_with('{') || !text.ends_with('}') || text.len() < 2 {
                return Err(MALFORMED);
            }
            if piece.operand >= operands {
                return Err(MALFORMED);
            }
            if piece.modifier != 0 {
                return Err(profile::MODIFIER);
            }
            next = end;
        }
    }
    Ok(())
}
/// Comments other than `#`, over the instruction's checked templates in
/// order, one step per template byte.
fn comments(
    rows: &s::Assembly,
    instruction: &s::AssemblyInstruction,
    work: &mut Work,
) -> Result<(), &'static str> {
    let mut comments = profile::Comments::default();
    for template in instruction.templates.slice(&rows.templates)? {
        work.charge(template.text.len())?;
        let spans = template
            .pieces
            .slice(&rows.pieces)?
            .iter()
            .map(|piece| (piece.start, piece.end));
        comments.template(&template.text, spans)?;
    }
    Ok(())
}
