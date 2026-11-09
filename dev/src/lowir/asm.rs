//! Inline assembly templates: typed operand kinds, literal text and operand uses.
use super::program::{Range, index};

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

/// One operand of a template. Every kind is a read-only register input.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A captured `ptr` bound to a general register other than rbx, rbp and rsp.
    Pointer,
    /// A `v128` whose `ptr` argument addresses its snapshot; the backend loads
    /// it into one of xmm0 to xmm15.
    Vector,
}
impl Kind {
    pub(super) const fn type_name(self) -> &'static str {
        match self {
            Self::Pointer => "ptr",
            Self::Vector => "v128",
        }
    }
    /// The input-storage flag: whether the argument is a pointer to the value.
    pub(super) const fn stored(self) -> u8 {
        match self {
            Self::Pointer => 0,
            Self::Vector => 1,
        }
    }
    pub(super) const fn registers(self) -> &'static [&'static str] {
        match self {
            Self::Pointer => &[
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
#[derive(Clone, Copy, Debug)]
pub(super) enum Piece {
    /// Literal assembly bytes in the shared text.
    Text(Range),
    /// The register bound to one operand, in its type's view.
    Use(u32),
}
#[derive(Clone, Copy, Debug)]
pub(super) struct Template {
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
    pub(super) operands: Vec<Kind>,
    pub(super) pieces: Vec<Piece>,
    pub(super) text: String,
}
impl Assemblies {
    pub(super) fn operands(&self, template: &Template) -> Result<&[Kind], &'static str> {
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
}
impl<'a> Draft<'a> {
    pub(super) fn new(assemblies: &'a mut Assemblies) -> Result<Self, &'static str> {
        Ok(Self {
            operands: index(assemblies.operands.len())?,
            pieces: index(assemblies.pieces.len())?,
            assemblies,
        })
    }
    /// # Errors
    /// Rejects an operand after the first piece and exhausted identities.
    pub fn operand(&mut self, kind: Kind) -> Result<(), &'static str> {
        if index(self.assemblies.pieces.len())? != self.pieces {
            return Err("LowIR assembly operand follows its template text");
        }
        index(self.assemblies.operands.len() + 1)?;
        self.assemblies.operands.push(kind);
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
        let declared = index(self.assemblies.operands.len())? - self.operands;
        if operand >= declared {
            return Err("LowIR assembly use names an undeclared operand");
        }
        index(self.assemblies.pieces.len() + 1)?;
        self.assemblies.pieces.push(Piece::Use(operand));
        Ok(())
    }
    /// # Errors
    /// Rejects bits outside the backend vocabulary and exhausted identities.
    pub fn finish(self, properties: u8, reads: u8, writes: u8) -> Result<AssemblyId, &'static str> {
        if properties & !PROPERTIES != 0 || (reads | writes) & !STATE != 0 {
            return Err("LowIR assembly header bits are outside the backend vocabulary");
        }
        let id = AssemblyId(index(self.assemblies.templates.len())?);
        self.assemblies.templates.push(Template {
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
