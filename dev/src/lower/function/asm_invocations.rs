//! The initial assembly profile: raw-pointer `reg` and 128-bit vector
//! `xmm_reg` inputs, Intel templates and the `nostack`/`preserves_flags`
//! options. Every other checked form fails here with its own text.
use super::{Lower, Selected};
use crate::{
    body::assembly::{self, Piece, Template},
    lower::context::Representation,
    lowir::{
        asm::{self as lowir, Draft, Kind},
        vocabulary::Type,
    },
    syntax::assembly::{self as syntax, Register},
    types::Kind as TypeKind,
};

pub(super) use crate::support::initial_assembly::OUTPUT;
use crate::support::initial_assembly::{GENERAL, XMM};
/// Status flags hold an undefined value on entry and DF is clear, so no
/// status read is needed; the template may depend on rounding control.
const READS: u8 = lowir::X87_CONTROL | lowir::MXCSR_CONTROL;
/// The x86 `preserves_flags` set: EFLAGS status, the x87 status word and the
/// MXCSR exception flags. DF must be clear on exit, so it is never written.
const STATUS: u8 = lowir::FLAGS | lowir::X87_STATUS | lowir::MXCSR_STATUS;

impl Lower<'_, '_, '_> {
    /// Lower one checked invocation to a fresh template and its `asm`.
    pub(super) fn assembly(&mut self, id: u32) -> Result<Selected, &'static str> {
        let body = self.body;
        let instruction = assembly::instruction(body, id)?;
        let data = body
            .assembly
            .as_deref()
            .ok_or("assembly effect has no retained body data")?;
        self.arguments.clear();
        assembly::profile::admit(data, instruction, |register, input| {
            let argument = self.assembly_input(register, input)?;
            self.arguments.push(argument);
            Ok(())
        })?;
        let operands = data
            .operands
            .get(instruction.operands.start as usize..instruction.operands.end as usize)
            .ok_or("assembly operand range is invalid")?;
        let templates = data
            .templates
            .get(instruction.templates.start as usize..instruction.templates.end as usize)
            .ok_or("assembly template range is invalid")?;
        let mut draft = self.context.output.assembly()?;
        for operand in operands {
            draft.operand(if operand.register == Register::Xmm {
                Kind::Vector
            } else {
                Kind::Pointer
            })?;
        }
        for (index, template) in templates.iter().enumerate() {
            if index != 0 {
                draft.text("\n")?;
            }
            text(&mut draft, template, pieces(&data.pieces, template)?)?;
        }
        let mut properties = lowir::SIDE_EFFECT;
        if instruction.options & syntax::NOSTACK == 0 {
            properties |= lowir::STACK | lowir::RED_ZONE;
        }
        let writes = if instruction.options & syntax::PRESERVES_FLAGS == 0 {
            STATUS
        } else {
            0
        };
        let template = draft.finish(properties, READS, writes)?;
        self.builder.asm(template, &self.arguments)?;
        Ok(Selected::Unit)
    }
    /// A raw pointer is captured by value; a vector passes its snapshot's address.
    fn assembly_input(
        &mut self,
        register: Register,
        input: crate::body::ir::ValueId,
    ) -> Result<crate::lowir::program::Value, &'static str> {
        let ty = self
            .body
            .values
            .get(input.0 as usize)
            .ok_or("assembly input lost its value")?
            .ty;
        let representation = self.context.representation(ty)?;
        match register {
            Register::General => {
                if !matches!(self.context.types.kind(ty), Some(TypeKind::Pointer { .. }))
                    || !matches!(representation, Representation::Scalar(Type::Ptr))
                {
                    return Err(GENERAL);
                }
                let value = self.value(input)?.runtime()?;
                if value.ty == Type::Ptr {
                    Ok(value)
                } else {
                    Err("assembly pointer input lost its scalar representation")
                }
            }
            Register::Xmm => {
                if !matches!(self.context.types.kind(ty), Some(TypeKind::Adt(..)))
                    || !matches!(representation, Representation::Object(span) if span.bytes == 16)
                {
                    return Err(XMM);
                }
                let Selected::Object(snapshot) = self.value(input)? else {
                    return Err("assembly vector input lacks its snapshot storage");
                };
                self.pointer(snapshot)
            }
            Register::Abcd | Register::Byte | Register::Explicit { .. } => {
                Err(assembly::profile::class(register).unwrap_or(GENERAL))
            }
        }
    }
}
fn pieces<'a>(all: &'a [Piece], template: &Template) -> Result<&'a [Piece], &'static str> {
    all.get(template.pieces.start as usize..template.pieces.end as usize)
        .ok_or("assembly piece range is invalid")
}
/// The template's literal text, with `{{` and `}}` unescaped, around its uses.
fn text(draft: &mut Draft<'_>, template: &Template, pieces: &[Piece]) -> Result<(), &'static str> {
    let mut cursor = 0;
    for piece in pieces {
        let start = piece.start as usize;
        if start < cursor {
            return Err("assembly placeholders lost their checked order");
        }
        literal(draft, template.text.get(cursor..start))?;
        draft.operand_use(piece.operand)?;
        cursor = piece.end as usize;
    }
    literal(draft, template.text.get(cursor..))
}
fn literal(draft: &mut Draft<'_>, text: Option<&str>) -> Result<(), &'static str> {
    let mut rest = text.ok_or("assembly placeholder lies outside its template")?;
    while let Some(at) = rest.find(['{', '}']) {
        let brace = &rest[at..=at];
        if rest.get(at + 1..at + 2) != Some(brace) {
            return Err("assembly template brace lost its checked escape");
        }
        draft.text(&rest[..=at])?;
        rest = &rest[at + 2..];
    }
    draft.text(rest)
}
