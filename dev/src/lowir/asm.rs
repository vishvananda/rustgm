//! Inline assembly templates: typed operand kinds, literal text and operand uses.
use super::{
    program::{Range, index},
    vocabulary::Type,
};

/// A program-wide template identity, in definition order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct AssemblyId(pub u32);

/// Property bit: the invocation has an observable side effect.
pub const SIDE_EFFECT: u8 = 1;
/// Property bit: the assembly may use the stack.
pub const STACK: u8 = 4;
/// Property bit: the assembly may write the red zone.
pub const RED_ZONE: u8 = 8;
/// State bit: the x86 status flags in `EFLAGS` (CF, PF, AF, ZF, SF, OF).
pub const FLAGS: u8 = 1;
/// State bit: the x87 status word.
pub const X87_STATUS: u8 = 2;
/// State bit: the `MXCSR` exception flags.
pub const MXCSR_STATUS: u8 = 4;
/// State bit: the x87 control word.
pub const X87_CONTROL: u8 = 8;
/// State bit: the `MXCSR` control fields.
pub const MXCSR_CONTROL: u8 = 16;
/// The backend vocabulary's widest property and state masks.
const PROPERTIES: u8 = 127;
const STATE: u8 = 63;

/// The backend value bound to one register operand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A captured `ptr` bound to a general register other than rbx, rbp and rsp.
    Pointer,
    /// A signed/unsigned 32-bit or raw 64-bit integer in a general register.
    Integer(Type),
    /// A `v128` whose `ptr` argument addresses its snapshot; the backend loads
    /// it into one of xmm0 to xmm15.
    Vector,
}
impl Kind {
    /// The backend value type of the operand, before storage indirection.
    #[must_use]
    pub const fn scalar_type(self) -> Type {
        match self {
            Self::Pointer => Type::Ptr,
            Self::Integer(ty) => ty,
            Self::Vector => Type::V128,
        }
    }
    pub(super) const fn type_name(self) -> &'static str {
        match self {
            Self::Pointer => "ptr",
            Self::Integer(ty) => ty.name(),
            Self::Vector => "v128",
        }
    }
    /// The input-storage flag: whether the argument is a pointer to the value.
    #[must_use]
    pub const fn stored(self) -> u8 {
        match self {
            Self::Pointer | Self::Integer(_) => 0,
            Self::Vector => 1,
        }
    }
    /// The registers the backend may bind, each a single-register group.
    #[must_use]
    pub const fn registers(self) -> &'static [&'static str] {
        match self {
            Self::Pointer | Self::Integer(_) => &[
                "rax", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12", "r13", "r14",
                "r15",
            ],
            Self::Vector => &[
                "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7", "xmm8", "xmm9",
                "xmm10", "xmm11", "xmm12", "xmm13", "xmm14", "xmm15",
            ],
        }
    }
}
/// A fixed general register admitted by Rust's x86-64 operand rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gpr {
    Ax,
    Cx,
    Dx,
    Si,
    Di,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
}
impl Gpr {
    /// # Errors
    /// Refuses reserved or unknown physical register numbers.
    pub fn from_base(base: u8) -> Result<Self, &'static str> {
        match base {
            0 => Ok(Self::Ax),
            1 => Ok(Self::Cx),
            2 => Ok(Self::Dx),
            6 => Ok(Self::Si),
            7 => Ok(Self::Di),
            8 => Ok(Self::R8),
            9 => Ok(Self::R9),
            10 => Ok(Self::R10),
            11 => Ok(Self::R11),
            12 => Ok(Self::R12),
            13 => Ok(Self::R13),
            14 => Ok(Self::R14),
            15 => Ok(Self::R15),
            _ => Err("unsupported LowIR fixed general register"),
        }
    }
    const fn registers(self) -> &'static [&'static str] {
        match self {
            Self::Ax => &["rax"],
            Self::Cx => &["rcx"],
            Self::Dx => &["rdx"],
            Self::Si => &["rsi"],
            Self::Di => &["rdi"],
            Self::R8 => &["r8"],
            Self::R9 => &["r9"],
            Self::R10 => &["r10"],
            Self::R11 => &["r11"],
            Self::R12 => &["r12"],
            Self::R13 => &["r13"],
            Self::R14 => &["r14"],
            Self::R15 => &["r15"],
        }
    }
}
/// One register operand and its positions in the invocation argument list.
#[derive(Clone, Copy, Debug)]
pub struct Binding {
    pub kind: Kind,
    pub fixed: Option<Gpr>,
    pub input: Option<u32>,
    pub output: Option<u32>,
    pub late: bool,
}
impl Binding {
    /// The singleton fixed register or the kind's selectable register groups.
    #[must_use]
    pub const fn registers(self) -> &'static [&'static str] {
        match self.fixed {
            Some(register) => register.registers(),
            None => self.kind.registers(),
        }
    }
    /// Backend operand read/write bits.
    #[must_use]
    pub const fn flags(self) -> u8 {
        (self.input.is_some() as u8) | ((self.output.is_some() as u8) << 1)
    }
}
#[derive(Clone, Copy, Debug)]
pub(super) enum Piece {
    /// Literal assembly bytes in the shared text.
    Text(Range),
    /// The register bound to one operand, in its type's view.
    Use { operand: u32, bits: u16 },
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Template {
    pub(super) att: bool,
    pub(super) nomem: bool,
    pub(super) arguments: u32,
    pub(super) properties: u8,
    pub(super) reads: u8,
    pub(super) writes: u8,
    pub(super) operands: Range,
    pub(super) pieces: Range,
}
/// Every template of one program. A program without assembly allocates none.
#[derive(Default)]
pub struct Assemblies {
    pub(super) templates: Vec<Template>,
    pub(super) operands: Vec<Binding>,
    pub(super) pieces: Vec<Piece>,
    pub(super) text: String,
}
/// One piece of a template as a serializer reads it.
#[derive(Clone, Copy, Debug)]
pub enum Part<'a> {
    /// Literal assembly text.
    Text(&'a str),
    /// The register bound to one operand of the template.
    Use { operand: u32, bits: u16 },
}
/// One defined template, read in definition order by a serializer.
#[derive(Clone, Copy)]
pub struct Reading<'a> {
    assemblies: &'a Assemblies,
    template: &'a Template,
}
impl<'a> Reading<'a> {
    /// The property, state-read and state-write bits of the header.
    #[must_use]
    pub const fn header(&self) -> (u8, u8, u8) {
        (
            self.template.properties,
            self.template.reads,
            self.template.writes,
        )
    }
    /// Dialect and memory-free effect of the template.
    #[must_use]
    pub const fn flags(&self) -> (bool, bool) {
        (self.template.att, self.template.nomem)
    }
    /// Number of input values and output storage addresses.
    #[must_use]
    pub const fn arguments(&self) -> u32 {
        self.template.arguments
    }
    /// # Errors
    /// Rejects a malformed operand range.
    pub fn operands(&self) -> Result<&'a [Binding], &'static str> {
        self.assemblies.operands(self.template)
    }
    /// # Errors
    /// Rejects malformed piece and text ranges.
    pub fn parts(&self) -> Result<Vec<Part<'a>>, &'static str> {
        self.assemblies
            .pieces(self.template)?
            .iter()
            .map(|piece| match *piece {
                Piece::Text(range) => self.assemblies.text(range).map(Part::Text),
                Piece::Use { operand, bits } => Ok(Part::Use { operand, bits }),
            })
            .collect()
    }
}
impl Assemblies {
    /// Every template in definition order.
    pub fn readings(&self) -> impl Iterator<Item = Reading<'_>> {
        self.templates.iter().map(|template| Reading {
            assemblies: self,
            template,
        })
    }
    pub(super) fn operands(&self, template: &Template) -> Result<&[Binding], &'static str> {
        self.operands
            .get(template.operands.start as usize..template.operands.end as usize)
            .ok_or("invalid LowIR assembly operand range")
    }
    pub(super) fn pieces(&self, template: &Template) -> Result<&[Piece], &'static str> {
        self.pieces
            .get(template.pieces.start as usize..template.pieces.end as usize)
            .ok_or("invalid LowIR assembly piece range")
    }
    pub(super) fn text(&self, range: Range) -> Result<&str, &'static str> {
        self.text
            .get(range.start as usize..range.end as usize)
            .ok_or("invalid LowIR assembly text range")
    }
    pub(super) fn template(&self, id: AssemblyId) -> Result<&Template, &'static str> {
        self.templates
            .get(id.0 as usize)
            .ok_or("invalid LowIR assembly template")
    }
}
/// A template under construction. Its operands precede its pieces; only a
/// finished draft defines a template.
pub struct Draft<'a> {
    assemblies: &'a mut Assemblies,
    operands: u32,
    pieces: u32,
    arguments: u32,
}
impl<'a> Draft<'a> {
    pub(super) fn new(assemblies: &'a mut Assemblies) -> Result<Self, &'static str> {
        Ok(Self {
            arguments: 0,
            operands: index(assemblies.operands.len())?,
            pieces: index(assemblies.pieces.len())?,
            assemblies,
        })
    }
    /// # Errors
    /// Rejects an operand after the first piece and exhausted identities.
    pub fn operand(&mut self, kind: Kind) -> Result<(), &'static str> {
        self.binding(kind, None, true, false, false)
    }
    /// Append an input, an output snapshot, or a read/write register operand.
    /// # Errors
    /// Rejects empty operands, exhausted identities and operands after text.
    pub fn binding(
        &mut self,
        kind: Kind,
        fixed: Option<Gpr>,
        input: bool,
        output: bool,
        late: bool,
    ) -> Result<(), &'static str> {
        if index(self.assemblies.pieces.len())? != self.pieces || (!input && !output) {
            return Err("invalid LowIR assembly operand or order");
        }
        if matches!(kind, Kind::Integer(ty) if !matches!(ty, Type::I32 | Type::U32 | Type::I64))
            || fixed.is_some() && kind == Kind::Vector
        {
            return Err("unsupported LowIR assembly register type");
        }
        index(self.assemblies.operands.len() + 1)?;
        let mut next = || -> Result<u32, &'static str> {
            let id = self.arguments;
            self.arguments = id
                .checked_add(1)
                .ok_or("LowIR assembly arguments exhausted")?;
            Ok(id)
        };
        let input = input.then(&mut next).transpose()?;
        let output = output.then(&mut next).transpose()?;
        self.assemblies.operands.push(Binding {
            kind,
            fixed,
            input,
            output,
            late,
        });
        Ok(())
    }
    /// Append literal text, extending an adjacent text piece.
    /// # Errors
    /// Rejects exhausted text identities.
    pub fn text(&mut self, text: &str) -> Result<(), &'static str> {
        if text.is_empty() {
            return Ok(());
        }
        let start = index(self.assemblies.text.len())?;
        let end = index(
            self.assemblies
                .text
                .len()
                .checked_add(text.len())
                .ok_or("LowIR assembly text exhausted")?,
        )?;
        self.assemblies.text.push_str(text);
        if index(self.assemblies.pieces.len())? > self.pieces
            && let Some(Piece::Text(range)) = self.assemblies.pieces.last_mut()
            && range.end == start
        {
            range.end = end;
            return Ok(());
        }
        index(self.assemblies.pieces.len() + 1)?;
        self.assemblies
            .pieces
            .push(Piece::Text(Range { start, end }));
        Ok(())
    }
    /// # Errors
    /// Rejects a use of an operand this template does not declare.
    pub fn operand_use(&mut self, operand: u32) -> Result<(), &'static str> {
        self.operand_use_view(operand, 0)
    }
    /// Use the default register view or its full 64-bit general-register view.
    /// # Errors
    /// Refuses stale operands, unsupported views and vector widening.
    pub fn operand_use_view(&mut self, operand: u32, bits: u16) -> Result<(), &'static str> {
        let declared = index(self.assemblies.operands.len())? - self.operands;
        if operand >= declared {
            return Err("LowIR assembly use names an undeclared operand");
        }
        index(self.assemblies.pieces.len() + 1)?;
        if !matches!(bits, 0 | 64)
            || bits != 0
                && self.assemblies.operands[(self.operands + operand) as usize].kind == Kind::Vector
        {
            return Err("unsupported LowIR assembly operand view");
        }
        self.assemblies.pieces.push(Piece::Use { operand, bits });
        Ok(())
    }
    /// # Errors
    /// Rejects bits outside the backend vocabulary and exhausted identities.
    pub fn finish(self, properties: u8, reads: u8, writes: u8) -> Result<AssemblyId, &'static str> {
        self.finish_flags(properties, reads, writes, false, false)
    }
    /// Finish a template with its dialect and exact memory effect.
    /// # Errors
    /// Rejects bits outside the backend vocabulary and exhausted identities.
    pub fn finish_flags(
        self,
        properties: u8,
        reads: u8,
        writes: u8,
        att: bool,
        nomem: bool,
    ) -> Result<AssemblyId, &'static str> {
        if properties & !PROPERTIES != 0 || (reads | writes) & !STATE != 0 {
            return Err("LowIR assembly header bits are outside the backend vocabulary");
        }
        let id = AssemblyId(index(self.assemblies.templates.len())?);
        self.assemblies.templates.push(Template {
            att,
            nomem,
            arguments: self.arguments,
            properties,
            reads,
            writes,
            operands: Range {
                start: self.operands,
                end: index(self.assemblies.operands.len())?,
            },
            pieces: Range {
                start: self.pieces,
                end: index(self.assemblies.pieces.len())?,
            },
        });
        Ok(id)
    }
}
