//! Checked pointer inputs, vector inputs/outputs and 32-/64-bit general registers.
//! Output snapshots precede the CFG's ordered destination writes; dialect and
//! memory effects remain explicit in each backend template.
use super::{Address, Lower, Selected};
use crate::{
    body::assembly::{self, Piece, Template},
    lower::context::Representation,
    lowir::{
        asm::{self as lowir, Draft, Gpr, Kind, Memory},
        vocabulary::Type,
    },
    syntax::assembly::{self as syntax, Register},
    types::Kind as TypeKind,
};

use crate::support::initial_assembly::OUTPUT;
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
        let mut bindings = Vec::new();
        assembly::profile::flags(data, instruction, |operand| {
            let ordinal =
                u32::try_from(bindings.len()).map_err(|_| "too many assembly operands")?;
            let (kind, fixed) = self.assembly_operand(id, ordinal, operand)?;
            bindings.push((
                kind,
                fixed,
                operand.input.is_some(),
                operand.output.is_some(),
                operand.late,
            ));
            Ok(())
        })?;
        let templates = data
            .templates
            .get(instruction.templates.start as usize..instruction.templates.end as usize)
            .ok_or("assembly template range is invalid")?;
        let mut draft = self.context.output.assembly()?;
        for (kind, fixed, input, output, late) in bindings {
            draft.binding(kind, fixed, input, output, late)?;
        }
        for (index, template) in templates.iter().enumerate() {
            if index != 0 {
                draft.text("\n")?;
            }
            text(&mut draft, template, pieces(&data.pieces, template)?)?;
        }
        let mut properties = if instruction.options & syntax::PURE == 0 {
            lowir::SIDE_EFFECT
        } else {
            lowir::DUPLICABLE
        };
        if instruction.options & syntax::NOSTACK == 0 {
            properties |= lowir::STACK | lowir::RED_ZONE;
        }
        let writes = if instruction.options & syntax::PRESERVES_FLAGS == 0 {
            STATUS
        } else {
            0
        };
        let memory = if instruction.options & syntax::NOMEM != 0 {
            Memory::None
        } else if instruction.options & syntax::READONLY != 0 {
            Memory::ReadOnly
        } else {
            Memory::ReadWrite
        };
        let template = draft.finish_memory(
            properties,
            READS,
            writes,
            instruction.options & syntax::ATT_SYNTAX != 0,
            memory,
        )?;
        self.builder.asm(template, &self.arguments)?;
        Ok(Selected::Unit)
    }
    fn assembly_operand(
        &mut self,
        id: u32,
        ordinal: u32,
        operand: &assembly::Operand,
    ) -> Result<(Kind, Option<Gpr>), &'static str> {
        let ty = operand.ty.ok_or("assembly operand lost its type")?;
        let kind = self.assembly_kind(operand.register, ty, operand.output.is_some())?;
        if let Some(input) = operand.input {
            let value = if matches!(kind, Kind::Integer(_)) {
                self.value(input)?.runtime()?
            } else {
                self.assembly_input(operand.register, input)?
            };
            self.arguments.push(value);
        }
        if operand.output.is_some() {
            let slot = match self.context.representation(ty)? {
                Representation::Scalar(native) => self.builder.slot(native)?,
                Representation::Object(span) if kind == Kind::Vector => {
                    self.builder.object_slot(span)?
                }
                _ => return Err(OUTPUT),
            };
            let pointer = self
                .builder
                .address(crate::lowir::program::Operand::Slot(slot))?;
            self.arguments.push(pointer);
            self.assembly_outputs
                .get_or_insert_with(Default::default)
                .insert((id, ordinal), pointer);
        }
        let fixed = match operand.register {
            Register::Explicit {
                base, xmm: false, ..
            } => Some(Gpr::from_base(base)?),
            _ => None,
        };
        Ok((kind, fixed))
    }
    fn assembly_kind(
        &mut self,
        register: Register,
        ty: crate::types::TypeId,
        output: bool,
    ) -> Result<Kind, &'static str> {
        if register == Register::Xmm {
            return if matches!(self.context.types.kind(ty), Some(TypeKind::Adt(..)))
                && matches!(self.context.representation(ty)?, Representation::Object(span) if span.bytes == 16)
            {
                Ok(Kind::Vector)
            } else {
                Err(XMM)
            };
        }
        if matches!(
            register,
            Register::General | Register::Explicit { xmm: false, .. }
        ) && let Some(TypeKind::Primitive(primitive)) = self.context.types.kind(ty)
            && let Some(bytes) = assembly::profile::integer_bytes(primitive)
        {
            if matches!(register, Register::Explicit { bytes: declared, .. } if declared != bytes) {
                return Err("assembly fixed register width differs from its integer operand");
            }
            let Representation::Scalar(native) = self.context.representation(ty)? else {
                return Err(GENERAL);
            };
            return Ok(Kind::Integer(native));
        }
        if output {
            return Err(OUTPUT);
        }
        Ok(Kind::Pointer)
    }
    pub(super) fn assembly_output(
        &mut self,
        effect: crate::body::ir::ValueId,
        operand: u32,
        ty: crate::types::TypeId,
    ) -> Result<Selected, &'static str> {
        let crate::body::ir::Rvalue::Assembly { instruction, .. } =
            self.body.values[effect.0 as usize].kind
        else {
            return Err("assembly output lost its effect");
        };
        let pointer = self
            .assembly_outputs
            .as_deref()
            .and_then(|outputs| outputs.get(&(instruction, operand)))
            .copied()
            .ok_or("assembly output lacks its native snapshot")?;
        match self.context.representation(ty)? {
            Representation::Scalar(native) => self
                .builder
                .load(native, pointer.operand)
                .map(Selected::Runtime),
            Representation::Object(span) => Ok(Selected::Object(Address {
                operand: pointer.operand,
                align: span.align,
            })),
            Representation::ScalarObject(_) => Err(OUTPUT),
        }
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
            Register::Abcd
            | Register::Byte
            | Register::Ymm
            | Register::Zmm
            | Register::Kreg
            | Register::Explicit { .. } => {
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
        let bits = match piece.modifier {
            b'e' => 32,
            b'r' => 64,
            _ => 0,
        };
        draft.operand_use_view(piece.operand, bits)?;
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
