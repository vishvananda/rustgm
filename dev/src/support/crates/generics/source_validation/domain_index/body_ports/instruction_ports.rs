//! Version thirteen's assembly columns are exact before any proof: dense
//! ordinals, partitioned ranges, inputs equal to the value's arguments,
//! well-formed placeholders and the initial assembly profile, whose comment
//! check comes last. The consumer still re-proves Copy inputs, CPU features
//! and lowering over its own body.
use super::{Context, DomainIndex, Work, g, s};
use crate::support::{crates::Scalar, initial_assembly as profile};

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
            instruction_profile(instruction, unsafe_, self.declaration_policies)?;
            let operands = instruction.operands.slice(&rows.operands)?;
            if self.declaration_policies {
                profile::effects(
                    instruction.options,
                    operands.iter().any(|operand| operand.output.is_some()),
                )?;
            }
            for operand in operands {
                self.types.root(self.headers, context, operand.ty, work)?;
            }
            placeholders(rows, instruction, self.declaration_policies, work)?;
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
        self.outputs(original, rows, work)?;
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
        let mut inputs = arguments.iter();
        let mut fixed = [0_u8; 16];
        for operand in operands {
            work.charge(1)?;
            if operand.output.is_some() && !self.declaration_policies {
                return Err(profile::OUTPUT);
            }
            if operand.input.is_none() && operand.output.is_none() {
                return Err("source assembly operand has neither input nor output");
            }
            if let Some(input) = operand.input {
                if inputs.next() != Some(&input) {
                    return Err("assembly input order differs from retained operands");
                }
                if original
                    .values
                    .get(input as usize)
                    .is_none_or(|value| value.ty != operand.ty)
                {
                    return Err("assembly input type differs from retained operand");
                }
            }
            if operand.late && operand.output.is_none() {
                return Err("source assembly input cannot be late");
            }
            if let Some(place) = operand.output {
                self.place(original, place, None, work)?;
                let ty = match place {
                    s::Place::Local(id) => original.locals[id as usize].ty,
                    s::Place::Projection(id) => original.projections[id as usize].ty,
                    s::Place::Temporary(id) => original.values[id as usize].ty,
                    s::Place::Static(id) => self.headers.static_header(id, work)?.ty,
                };
                if ty != operand.ty {
                    return Err("assembly output type differs from retained place");
                }
            }
            self.class(operand)?;
            fixed_alias(operand, &mut fixed)?;
        }
        if inputs.next().is_some() {
            return Err("assembly input order differs from retained operands");
        }
        Ok(())
    }
    fn outputs(
        &self,
        original: &s::Original,
        rows: &s::Assembly,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        // The legacy vocabulary and input-only pass already exclude output nodes.
        if !self.declaration_policies {
            return Ok(());
        }
        let mut stores = crate::support::hash::HashMap::default();
        for statement in &original.statements {
            work.charge(1)?;
            if let s::Action::Store { place, value } = statement.action
                && matches!(
                    original.values.get(value as usize).map(|value| value.kind),
                    Some(s::ValueKind::AssemblyOutput { .. })
                )
                && stores.insert(value, place).is_some()
            {
                return Err("source assembly output has repeated stores");
            }
        }
        let mut outputs = vec![false; rows.operands.len()];
        for (index, value) in original.values.iter().enumerate() {
            let s::ValueKind::AssemblyOutput { effect, operand } = value.kind else {
                continue;
            };
            work.charge(1)?;
            let Some(s::Value {
                kind: s::ValueKind::Assembly { instruction, .. },
                ..
            }) = original.values.get(effect as usize)
            else {
                return Err("assembly output names another value kind");
            };
            let range = rows.instructions[*instruction as usize].operands;
            let operands = range.slice(&rows.operands)?;
            let output = operands
                .get(operand as usize)
                .ok_or("assembly output operand is stale")?;
            if output.output.is_none() || output.ty != value.ty || effect as usize >= index {
                return Err("assembly output differs from its retained typed place");
            }
            let emitted = &mut outputs[range.start as usize + operand as usize];
            if *emitted
                || stores
                    .get(&u32::try_from(index).map_err(|_| "source value overflow")?)
                    .copied()
                    != output.output
            {
                return Err("source assembly output differs from its ordered store");
            }
            *emitted = true;
        }
        if outputs
            .iter()
            .zip(&rows.operands)
            .any(|(&emitted, operand)| emitted != operand.output.is_some())
        {
            return Err("source assembly output column differs from its output values");
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
                Some(s::Type::Scalar(
                    crate::support::crates::Scalar::I32
                    | crate::support::crates::Scalar::U32
                    | crate::support::crates::Scalar::I64
                    | crate::support::crates::Scalar::U64
                    | crate::support::crates::Scalar::Isize
                    | crate::support::crates::Scalar::Usize,
                )) if self.declaration_policies => Ok(()),
                Some(s::Type::RawPointer { pointee, .. })
                    if !matches!(ty(*pointee), Some(s::Type::Slice(_) | s::Type::Str) | None) =>
                {
                    if operand.output.is_some() {
                        Err(profile::OUTPUT)
                    } else {
                        Ok(())
                    }
                }
                _ => Err(profile::GENERAL),
            },
            s::AssemblyClass::Vector => match ty(operand.ty) {
                Some(s::Type::Nominal { .. })
                    if operand.output.is_none() || self.declaration_policies =>
                {
                    Ok(())
                }
                _ => Err(profile::XMM),
            },
            s::AssemblyClass::Ymm | s::AssemblyClass::Zmm => {
                let form = if operand.class == s::AssemblyClass::Ymm {
                    profile::YMM
                } else {
                    profile::ZMM
                };
                if self.declaration_policies
                    && matches!(ty(operand.ty), Some(s::Type::Nominal { .. }))
                {
                    Ok(())
                } else {
                    Err(form)
                }
            }
            s::AssemblyClass::Kreg => {
                if !self.declaration_policies {
                    return Err(profile::KREG);
                }
                if operand.output.is_some() {
                    return Err(profile::OUTPUT);
                }
                match ty(operand.ty) {
                    Some(s::Type::Scalar(
                        Scalar::I8
                        | Scalar::U8
                        | Scalar::I16
                        | Scalar::U16
                        | Scalar::I32
                        | Scalar::U32
                        | Scalar::I64
                        | Scalar::U64
                        | Scalar::Isize
                        | Scalar::Usize,
                    )) => Ok(()),
                    _ => Err(profile::KREG),
                }
            }
            s::AssemblyClass::Abcd => Err(profile::ABCD),
            s::AssemblyClass::Byte => {
                if self.declaration_policies
                    && matches!(
                        ty(operand.ty),
                        Some(s::Type::Scalar(Scalar::I8 | Scalar::U8))
                    )
                {
                    Ok(())
                } else {
                    Err(profile::BYTE)
                }
            }
            s::AssemblyClass::Explicit { base, bytes, xmm } => {
                if !self.declaration_policies {
                    return Err(profile::EXPLICIT);
                }
                profile::fixed_register(base, bytes, xmm)?;
                let actual = match ty(operand.ty) {
                    Some(s::Type::Scalar(Scalar::I32 | Scalar::U32)) => Some(4),
                    Some(s::Type::Scalar(
                        Scalar::I64 | Scalar::U64 | Scalar::Isize | Scalar::Usize,
                    )) => Some(8),
                    _ => None,
                };
                if actual == Some(bytes) {
                    Ok(())
                } else {
                    Err("assembly fixed register width differs from its integer operand")
                }
            }
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
fn instruction_profile(
    row: &s::AssemblyInstruction,
    unsafe_: bool,
    flags: bool,
) -> Result<(), &'static str> {
    if row.options & !profile::KNOWN != 0 {
        return Err("unknown source assembly option");
    }
    if flags {
        profile::flags_option(row.options)?;
    } else {
        profile::option(row.options)?;
    }
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
/// template, each naming an operand with its exact admitted register view.
fn placeholders(
    rows: &s::Assembly,
    instruction: &s::AssemblyInstruction,
    extended: bool,
    work: &mut Work,
) -> Result<(), &'static str> {
    const MALFORMED: &str = "source assembly placeholder is malformed";
    let operands = instruction.operands.slice(&rows.operands)?;
    for template in instruction.templates.slice(&rows.templates)? {
        let mut next = 0_usize;
        for piece in template.pieces.slice(&rows.pieces)? {
            work.charge(1)?;
            let (start, end) = (piece.start as usize, piece.end as usize);
            let text = template.text.get(start..end).ok_or(MALFORMED)?;
            if start < next || !text.starts_with('{') || !text.ends_with('}') || text.len() < 2 {
                return Err(MALFORMED);
            }
            let operand = operands.get(piece.operand as usize).ok_or(MALFORMED)?;
            if matches!(operand.class, s::AssemblyClass::Explicit { .. }) {
                return Err("source assembly explicit register is named by a placeholder");
            }
            if piece.modifier != 0
                && !(extended
                    && matches!(piece.modifier, b'e' | b'r')
                    && operand.class == s::AssemblyClass::General)
            {
                return Err(profile::MODIFIER);
            }
            let written = text[1..text.len() - 1]
                .rsplit_once(':')
                .map(|(_, suffix)| suffix.trim());
            let expected = match piece.modifier {
                b'e' => Some("e"),
                b'r' => Some("r"),
                _ => None,
            };
            if written != expected {
                return Err(MALFORMED);
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

/// Check physical aliases once, including a late output's allowed input overlap.
fn fixed_alias(operand: &s::AssemblyOperand, seen: &mut [u8; 16]) -> Result<(), &'static str> {
    let s::AssemblyClass::Explicit { base, .. } = operand.class else {
        return Ok(());
    };
    let previous = &mut seen[base as usize]; // The class check admitted the base.
    let reads = operand.input.is_some();
    let writes = operand.output.is_some();
    if reads && *previous & (1 | 4) != 0
        || writes && (*previous & 2 != 0 || !operand.late && *previous & 1 != 0)
    {
        return Err("source assembly operands overlap the same explicit register");
    }
    *previous |=
        u8::from(reads) | (u8::from(writes) << 1) | (u8::from(writes && !operand.late) << 2);
    Ok(())
}
