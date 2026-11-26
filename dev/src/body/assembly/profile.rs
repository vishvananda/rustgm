//! The initial assembly profile and its integer-register extension share
//! one check order over a body's payload. Its texts
//! are `support::initial_assembly`'s, which every reader shares as well.
use super::{Instruction, Payload, Template};
use crate::{
    body::ir::ValueId,
    support::initial_assembly::{
        ABCD, BYTE, Comments, EXPLICIT, KREG, MODIFIER, OUTPUT, YMM, ZMM, option,
    },
    syntax::assembly::Register,
};

/// The text of a register class the profile never admits.
pub(crate) const fn class(register: Register) -> Option<&'static str> {
    match register {
        Register::Abcd => Some(ABCD),
        Register::Byte => Some(BYTE),
        Register::Explicit { .. } => Some(EXPLICIT),
        Register::Ymm => Some(YMM),
        Register::Zmm => Some(ZMM),
        Register::Kreg => Some(KREG),
        Register::General | Register::Xmm => None,
    }
}
/// Admit one instruction in the profile's order: options in bit order, then
/// each operand in operand order (its output first, then its class), then
/// the modifiers, then comments other than `#` over the joined templates.
/// `input` decides each `reg` or `xmm_reg` input's type.
pub(crate) fn admit(
    data: &Payload,
    instruction: &Instruction,
    mut input: impl FnMut(Register, ValueId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    checked(data, instruction, false, |operand| {
        if operand.discarded_output {
            return Err(OUTPUT);
        }
        let (Some(value), None) = (operand.input, operand.output) else {
            return Err(OUTPUT);
        };
        input(operand.register, value)
    })
}
/// The flags extension retains integer register inputs and ordered outputs.
pub(crate) fn flags(
    data: &Payload,
    instruction: &Instruction,
    operand: impl FnMut(&super::Operand) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    let mut operand = operand;
    checked(data, instruction, true, |value| {
        if value.discarded_output || value.input.is_none() && value.output.is_none() {
            return Err(OUTPUT);
        }
        operand(value)
    })
}
fn checked(
    data: &Payload,
    instruction: &Instruction,
    flags: bool,
    mut check: impl FnMut(&super::Operand) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    if flags {
        crate::support::initial_assembly::flags_option(instruction.options)?;
    } else {
        option(instruction.options)?;
    }
    let operands = data
        .operands
        .get(instruction.operands.start as usize..instruction.operands.end as usize)
        .ok_or("assembly operand range is invalid")?;
    for operand in operands {
        check(operand)?;
        if !(flags && matches!(operand.register, Register::Explicit { xmm: false, .. }))
            && let Some(form) = class(operand.register)
        {
            return Err(form);
        }
    }
    let templates = data
        .templates
        .get(instruction.templates.start as usize..instruction.templates.end as usize)
        .ok_or("assembly template range is invalid")?;
    let pieces = |template: &Template| {
        data.pieces
            .get(template.pieces.start as usize..template.pieces.end as usize)
            .ok_or("assembly piece range is invalid")
    };
    for template in templates {
        if pieces(template)?.iter().any(|piece| {
            piece.modifier != 0
                && !(flags
                    && piece.modifier == b'r'
                    && operands
                        .get(piece.operand as usize)
                        .is_some_and(|operand| operand.register == Register::General))
        }) {
            return Err(MODIFIER);
        }
    }
    let mut comments = Comments::default();
    for template in templates {
        let spans = pieces(template)?
            .iter()
            .map(|piece| (piece.start, piece.end));
        comments.template(&template.text, spans)?;
    }
    Ok(())
}

/// Widths admitted by the integer-register extension, on the course target.
pub(crate) const fn integer_bytes(primitive: crate::types::Primitive) -> Option<u8> {
    use crate::types::Primitive;
    match primitive {
        Primitive::I32 | Primitive::U32 => Some(4),
        Primitive::I64 | Primitive::U64 | Primitive::Isize | Primitive::Usize => Some(8),
        _ => None,
    }
}
