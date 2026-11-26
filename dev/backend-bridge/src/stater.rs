//! The module-level statements, in the order the pinned reader makes them
//! from `RustGM`'s text: globals with the symbols their initializers name,
//! then assembly templates, then each function's symbol, signature, facts
//! and body.
//!
//! A symbol is interned where the text first names it, so its identity is
//! the reader's. A signature is stated where the text states one: each
//! function header and each call through a value (`as (...)`), never shared.
use crate::{
    Failure, Forms, ffi, instructions,
    literals::{self, Context},
    records::{
        ARITY_VARIADIC, ASM_BANK_GPR, ASM_BANK_VECTOR, ASM_LOCATION_REGISTER, ASM_MEMORY_NONE,
        ASM_MEMORY_READ, ASM_MEMORY_READ_WRITE, ASM_OPERAND_READ, ASM_OPERAND_WRITE,
        ASM_PIECE_OPERAND, ASM_PIECE_TEXT, AsmAlternative, AsmGroup, AsmOperand, AsmPiece,
        AsmRegister, BINDING_INTERNAL, BINDING_WEAK, Boundary, CLASS_MEMORY, CLASS_STATED,
        DATA_ADDRESS, DATA_SCALAR, DATA_ZERO, DataItem, EIGHTBYTE_INTEGER, EIGHTBYTE_NONE,
        EIGHTBYTE_SSE, EIGHTBYTE_SSEUP, KEY_BINDING, KEY_LINKAGE, KEY_ROLE, LINKAGE_DEFAULT,
        Literal, NO_ID, Parameter, RETURN_NORETURN, ROLE_ENTRY, STORAGE_DEFAULT, STORAGE_READONLY,
        STORAGE_THREAD_LOCAL, TYPE_F32, TYPE_F64, TYPE_I1, TYPE_I8, TYPE_I16, TYPE_I32, TYPE_I64,
        TYPE_I128, TYPE_PTR, TYPE_U8, TYPE_U16, TYPE_U32, TYPE_V128, TYPE_VOID, Type, object,
    },
};
use rustgm::lowir::{
    asm::{Kind, Memory, Part},
    boundary::{self, Class, Classes},
    data::{GlobalId, GlobalKind, GlobalStorage, Item, Target},
    program::{
        Binding, Literal as Constant, MemorySpan, Program, Signature, SignatureId, SymbolId,
    },
    vocabulary::Type as Scalar,
};

/// The type word of a scalar.
pub(crate) fn scalar(ty: Scalar) -> Type {
    match ty {
        Scalar::Void => TYPE_VOID,
        Scalar::I1 => TYPE_I1,
        Scalar::I8 => TYPE_I8,
        Scalar::U8 => TYPE_U8,
        Scalar::I16 => TYPE_I16,
        Scalar::U16 => TYPE_U16,
        Scalar::I32 => TYPE_I32,
        Scalar::U32 => TYPE_U32,
        Scalar::I64 => TYPE_I64,
        Scalar::I128 => TYPE_I128,
        Scalar::F32 => TYPE_F32,
        Scalar::F64 => TYPE_F64,
        Scalar::Ptr => TYPE_PTR,
        Scalar::V128 => TYPE_V128,
    }
}

fn memory(effect: Memory) -> u8 {
    match effect {
        Memory::None => ASM_MEMORY_NONE,
        Memory::ReadOnly => ASM_MEMORY_READ,
        Memory::ReadWrite => ASM_MEMORY_READ_WRITE,
    }
}

/// The log2 alignment of a span, or the form when it is not a power of two.
pub(crate) fn alignment(span: MemorySpan) -> Result<u32, &'static str> {
    if span.align.is_power_of_two() {
        Ok(span.align.trailing_zeros())
    } else {
        Err("object alignment that is not a power of two")
    }
}

/// The classes byte of `classes=` as the reader spells it for an object.
pub(crate) fn classes(value: Classes) -> u8 {
    let eightbyte = |class| match class {
        Class::None => EIGHTBYTE_NONE,
        Class::Integer => EIGHTBYTE_INTEGER,
        Class::Sse => EIGHTBYTE_SSE,
        Class::SseUp => EIGHTBYTE_SSEUP,
    };
    match value {
        Classes::Memory => CLASS_STATED | CLASS_MEMORY,
        Classes::Eightbytes { first, second } => {
            CLASS_STATED | eightbyte(first) | (eightbyte(second.unwrap_or(Class::None)) << 2)
        }
    }
}

/// A register of an assembly operand, by bank and number.
fn register(name: &str) -> Option<AsmRegister> {
    const GENERAL: [&str; 16] = [
        "rax", "rcx", "rdx", "rbx", "rsp", "rbp", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12",
        "r13", "r14", "r15",
    ];
    let (bank, number) = match GENERAL.iter().position(|known| *known == name) {
        Some(number) => (ASM_BANK_GPR, u8::try_from(number).ok()?),
        None => (
            ASM_BANK_VECTOR,
            name.strip_prefix("xmm")?
                .parse::<u8>()
                .ok()
                .filter(|n| *n < 16)?,
        ),
    };
    // `(name:0)`: the stated width is 0 and the offset absent.
    Some(AsmRegister {
        bank,
        number,
        ..AsmRegister::default()
    })
}

fn count(length: usize) -> Result<u32, Failure> {
    u32::try_from(length).map_err(|_| Failure::from("LowIR identity space exhausted"))
}

/// The module being stated, with the identities it has interned.
pub(crate) struct Stater<'p> {
    pub(crate) program: &'p Program,
    pub(crate) module: ffi::Module,
    /// The library identity of each function and global, once named.
    functions: Vec<u32>,
    globals: Vec<u32>,
    /// The library identity and operand count of each template.
    pub(crate) templates: Vec<(u32, usize)>,
    pub(crate) forms: Forms,
    parameters: Vec<Parameter>,
}
impl<'p> Stater<'p> {
    pub(crate) fn new(program: &'p Program, module: ffi::Module) -> Self {
        Self {
            program,
            module,
            functions: vec![NO_ID; program.symbols.len()],
            globals: vec![NO_ID; program.globals.len()],
            templates: Vec::new(),
            forms: Forms::default(),
            parameters: Vec::new(),
        }
    }
    /// The library identity of a function, interned when first named.
    pub(crate) fn function_symbol(&mut self, id: SymbolId) -> Result<u32, Failure> {
        let known = self
            .functions
            .get_mut(id.0 as usize)
            .ok_or("invalid LowIR symbol")?;
        if *known == NO_ID {
            *known = self
                .module
                .symbol(&self.program.symbols[id.0 as usize].name)?;
        }
        Ok(*known)
    }
    /// The library identity of a global, interned when first named.
    pub(crate) fn global_symbol(&mut self, id: GlobalId) -> Result<u32, Failure> {
        let known = self
            .globals
            .get_mut(id.0 as usize)
            .ok_or("invalid LowIR data address target")?;
        if *known == NO_ID {
            *known = self
                .module
                .symbol(&self.program.globals[id.0 as usize].name)?;
        }
        Ok(*known)
    }
    /// The type word of an object of `span`.
    pub(crate) fn object(&mut self, span: MemorySpan) -> Type {
        match alignment(span) {
            Ok(log2) => object(span.bytes, log2),
            Err(form) => {
                self.forms.note(form);
                TYPE_VOID
            }
        }
    }
    fn boundary_type(&mut self, ty: boundary::Type) -> (Type, u8) {
        match ty {
            boundary::Type::Scalar(ty) => (scalar(ty), 0),
            boundary::Type::Object(value) => {
                // The text states classes of a direct object of at most 16 bytes.
                let stated = if value.span.bytes <= 16 {
                    classes(value.classes)
                } else {
                    0
                };
                (self.object(value.span), stated)
            }
        }
    }
    /// The type word of a signature's result.
    pub(crate) fn result(&mut self, id: SignatureId) -> Result<Type, Failure> {
        let result = self.program.result_type(id)?;
        Ok(self.boundary_type(result).0)
    }
    /// State one signature as the text spells it; `NO_ID` when it holds a
    /// form the bridge does not state.
    pub(crate) fn boundary(&mut self, id: SignatureId) -> Result<u32, Failure> {
        let program = self.program;
        let noted = self.forms.total();
        let mut parameters = std::mem::take(&mut self.parameters);
        parameters.clear();
        let (result, variadic, noreturn) = match program.signatures.get(id.0 as usize) {
            Some(Signature::Scalar(signature)) => {
                parameters.extend(program.parameters(id)?.iter().map(|&ty| Parameter {
                    ty: scalar(ty),
                    ..Parameter::default()
                }));
                (
                    boundary::Type::Scalar(signature.result),
                    signature.variadic,
                    signature.noreturn,
                )
            }
            Some(Signature::Object(_)) => {
                let signature = program.object_boundary(id)?;
                for &parameter in &signature.parameters {
                    let (ty, classes) = self.boundary_type(parameter);
                    parameters.push(Parameter {
                        ty,
                        classes,
                        ..Parameter::default()
                    });
                }
                (signature.result, signature.variadic, signature.noreturn)
            }
            None => return Err("invalid LowIR signature".into()),
        };
        let (result, result_classes) = self.boundary_type(result);
        let facts = Boundary {
            arity: if variadic { ARITY_VARIADIC } else { 0 },
            returns: if noreturn { RETURN_NORETURN } else { 0 },
            result_classes,
            ..Boundary::default()
        };
        let stated = if self.forms.total() == noted {
            self.module.boundary(result, &parameters, &facts)
        } else {
            Ok(NO_ID)
        };
        self.parameters = parameters;
        Ok(stated?)
    }
    /// Every global in program order, after the symbols its data names.
    pub(crate) fn globals(&mut self) -> Result<(), Failure> {
        let program = self.program;
        let mut items = Vec::new();
        for (index, global) in program.globals.iter().enumerate() {
            let id = GlobalId(count(index)?);
            if program.init_array_section(id).is_some() {
                self.forms.note("GNU init-array section");
                continue;
            }
            let noted = self.forms.total();
            let symbol = self.global_symbol(id)?;
            let ty = self.object(global.span);
            items.clear();
            let declaration = global.kind == GlobalKind::Declaration;
            if !declaration {
                let range = global.items.ok_or("uninitialized LowIR global")?;
                let data = program
                    .data
                    .get(range.start as usize..range.end as usize)
                    .ok_or("invalid LowIR global item range")?;
                for &item in data {
                    items.push(self.item(item)?);
                }
            }
            let storage = match global.storage {
                GlobalStorage::Writable => STORAGE_DEFAULT,
                GlobalStorage::Readonly => STORAGE_READONLY,
                GlobalStorage::ThreadLocal => STORAGE_THREAD_LOCAL,
            };
            if self.forms.total() == noted {
                self.module
                    .global(symbol, ty, storage, &items, declaration)?;
                self.binding(symbol, global.native_binding())?;
            }
        }
        Ok(())
    }
    fn item(&mut self, item: Item) -> Result<DataItem, Failure> {
        let mut data = DataItem {
            symbol: NO_ID,
            ..DataItem::default()
        };
        match item {
            Item::Zero(bytes) => {
                data.kind = DATA_ZERO;
                data.amount = i64::try_from(bytes).unwrap_or_else(|_| {
                    self.forms.note("zero data beyond 2^63 bytes");
                    0
                });
            }
            Item::Bits { ty, bits } => {
                data.kind = DATA_SCALAR;
                data.ty = scalar(ty);
                match literals::read(Constant::Integer(bits), Context::of(ty)) {
                    Ok(read) => data.bits = read.bits,
                    Err(form) => {
                        self.forms.note(form);
                        data.bits = Literal::default();
                    }
                }
            }
            Item::Address { target, addend } => {
                data.kind = DATA_ADDRESS;
                data.ty = TYPE_PTR;
                data.amount = addend;
                data.symbol = match target {
                    Target::Global(id) => self.global_symbol(id)?,
                    Target::Function(id) => self.function_symbol(id)?,
                };
            }
        }
        Ok(data)
    }
    /// Every assembly template in definition order: `assembly @asmN`.
    pub(crate) fn templates(&mut self) -> Result<(), Failure> {
        let Some(assemblies) = self.program.assemblies() else {
            return Ok(());
        };
        for reading in assemblies.readings() {
            let noted = self.forms.total();
            let (properties, reads, writes) = reading.header();
            let kinds = reading.operands()?;
            let mut operands = Vec::with_capacity(kinds.len());
            let mut alternatives = Vec::with_capacity(kinds.len());
            let (mut groups, mut registers) = (Vec::new(), Vec::new());
            for (index, &binding) in kinds.iter().enumerate() {
                let kind = binding.kind;
                let first_group = count(groups.len())?;
                for name in binding.registers() {
                    let Some(register) = register(name) else {
                        self.forms
                            .note("assembly register the bridge does not name");
                        continue;
                    };
                    groups.push(AsmGroup {
                        first_register: count(registers.len())?,
                        register_count: 1,
                        reserved: 0,
                    });
                    registers.push(register);
                }
                // `alternative 1 0 -1`: in a register, no early write, no match.
                alternatives.push(AsmAlternative {
                    first_group,
                    matching: NO_ID,
                    group_count: u16::try_from(groups.len() - first_group as usize)
                        .map_err(|_| Failure::from("too many LowIR assembly registers"))?,
                    locations: ASM_LOCATION_REGISTER,
                    early_write: u8::from(binding.output.is_some() && !binding.late),
                    ..AsmAlternative::default()
                });
                // `operand T input -1 1 stored 0`: read only, one alternative.
                operands.push(AsmOperand {
                    ty: match kind {
                        Kind::Pointer => TYPE_PTR,
                        Kind::Vector => TYPE_V128,
                        Kind::Integer(ty) => scalar(ty),
                    },
                    input: binding.input.unwrap_or(NO_ID),
                    output: binding.output.unwrap_or(NO_ID),
                    first_alternative: count(index)?,
                    alternative_count: 1,
                    flags: operand_flags(binding),
                    input_storage: binding.input_storage(),
                    ..AsmOperand::default()
                });
            }
            let (mut text, mut pieces) = (Vec::new(), Vec::new());
            for part in reading.parts()? {
                pieces.push(match part {
                    Part::Text(literal) => {
                        if literal.contains('\0') {
                            self.forms.note("assembly text with a NUL byte");
                        }
                        let first = count(text.len())?;
                        text.extend_from_slice(literal.as_bytes());
                        AsmPiece {
                            kind: ASM_PIECE_TEXT,
                            operand: NO_ID,
                            first,
                            count: count(literal.len())?,
                            ..AsmPiece::default()
                        }
                    }
                    Part::Use { operand, bits } => AsmPiece {
                        kind: ASM_PIECE_OPERAND,
                        operand,
                        bits,
                        ..AsmPiece::default()
                    },
                });
            }
            if self.forms.total() != noted {
                self.templates.push((NO_ID, reading.arguments() as usize));
                continue;
            }
            let id = self.module.assembly(&ffi::Template {
                att: reading.flags().0,
                memory: memory(reading.memory()),
                arguments: reading.arguments(),
                properties: u16::from(properties),
                reads: u32::from(reads),
                writes: u32::from(writes),
                text: &text,
                operands: &operands,
                alternatives: &alternatives,
                groups: &groups,
                registers: &registers,
                pieces: &pieces,
            })?;
            self.templates.push((id, reading.arguments() as usize));
        }
        Ok(())
    }
    /// Every function in program order: its symbol, its header signature,
    /// the default linkage and role the text states, then its body.
    pub(crate) fn functions(&mut self) -> Result<(), Failure> {
        let program = self.program;
        let mut scratch = instructions::Scratch::default();
        for (index, symbol) in program.symbols.iter().enumerate() {
            let id = self.function_symbol(SymbolId(count(index)?))?;
            let signature = self.boundary(symbol.signature)?;
            let declared = signature != NO_ID;
            if declared {
                self.module.function(id, signature, symbol.body.is_some())?;
                self.module.symbol_fact(id, KEY_LINKAGE, LINKAGE_DEFAULT)?;
                self.binding(id, symbol.native_binding())?;
                if symbol.entry {
                    self.module.symbol_fact(id, KEY_ROLE, ROLE_ENTRY)?;
                }
            }
            if let Some(function) = &symbol.body {
                let body = instructions::Owner {
                    symbol: id,
                    signature: symbol.signature,
                    declared,
                };
                instructions::state(self, &mut scratch, body, function)?;
            }
        }
        Ok(())
    }
    fn binding(&mut self, symbol: u32, binding: Binding) -> Result<(), Failure> {
        let value = match binding {
            Binding::Strong => return Ok(()),
            Binding::Weak => BINDING_WEAK,
            Binding::Internal => BINDING_INTERNAL,
        };
        Ok(self.module.symbol_fact(symbol, KEY_BINDING, value)?)
    }

    /// The module, unless it holds a form the bridge does not state.
    pub(crate) fn finish(self) -> Result<ffi::Module, Failure> {
        if self.forms.is_empty() {
            Ok(self.module)
        } else {
            Err(Failure::Unsupported(self.forms))
        }
    }
}

/// Map the model's direction to the pinned library's operand vocabulary.
fn operand_flags(binding: rustgm::lowir::asm::Binding) -> u8 {
    (u8::from(binding.input.is_some()) * ASM_OPERAND_READ)
        | (u8::from(binding.output.is_some()) * ASM_OPERAND_WRITE)
}
