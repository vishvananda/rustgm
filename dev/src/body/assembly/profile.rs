//! The initial assembly profile's one check order over a body's payload,
//! shared by native lowering and version-thirteen source export. Its texts
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
    option(instruction.options)?;
    let operands = data
        .operands
        .get(instruction.operands.start as usize..instruction.operands.end as usize)
        .ok_or("assembly operand range is invalid")?;
    for operand in operands {
        let (Some(value), None) = (operand.input, operand.output) else {
            return Err(OUTPUT);
        };
        if let Some(form) = class(operand.register) {
            return Err(form);
        }
        input(operand.register, value)?;
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
        if pieces(template)?.iter().any(|piece| piece.modifier != 0) {
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
