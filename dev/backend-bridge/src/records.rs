//! The records and numbers of the pinned CPPGM `LowIR` library, interface
//! version 5. This bridge targets x86-64 Linux. `RustGM`'s model supplies
//! explicit object classes, as its text does.
//!
//! Each record is the fixed-width C projection declared by the pinned
//! `lowir_records.h` and `lowir_interface.h`; each number is an enumerator of
//! the pinned `lowir_c_vocabulary.h` or `lowir_interface.h`. The sizes below
//! are checked at compile time against the sizes the pinned headers give a C
//! compiler; the field order is checked by the handshake, whose module must
//! read back from the library's own text as the same object.
use std::mem::size_of;

/// The only interface version this bridge states records for.
pub const ABI_VERSION: u32 = 5;
/// `CPPGM_NO_ID`: the absent identity.
pub const NO_ID: u32 = u32::MAX;
/// `CPPGM_TARGET_LINUX`: an ELF object for x86-64 Linux.
pub const TARGET_LINUX: u32 = 0;

/// A type word: kind in bits 0 to 7, log2 alignment in bits 8 to 15 and the
/// byte size above. A scalar is its kind alone; `ptr` with size 0 is 8 bytes.
pub type Type = u64;
pub const TYPE_VOID: Type = 0;
pub const TYPE_I1: Type = 1;
pub const TYPE_I8: Type = 2;
pub const TYPE_U8: Type = 3;
pub const TYPE_I16: Type = 4;
pub const TYPE_U16: Type = 5;
pub const TYPE_I32: Type = 6;
pub const TYPE_U32: Type = 7;
pub const TYPE_I64: Type = 8;
pub const TYPE_I128: Type = 9;
pub const TYPE_F32: Type = 10;
pub const TYPE_F64: Type = 11;
pub const TYPE_PTR: Type = 13;
pub const TYPE_V128: Type = 14;
pub const TYPE_OBJ: Type = 15;
/// The type kinds a float literal's operand names as its `float_kind`.
pub const KIND_F32: u8 = 10;
pub const KIND_F64: u8 = 11;

pub const OP_COPY: u8 = 1;
pub const OP_ADDR: u8 = 3;
pub const OP_LOAD: u8 = 4;
pub const OP_ATOMIC_LOAD: u8 = 5;
pub const OP_INDEX: u8 = 6;
pub const OP_UNARY: u8 = 7;
pub const OP_BINARY: u8 = 8;
pub const OP_COMPARE: u8 = 9;
pub const OP_CONVERT: u8 = 11;
pub const OP_VECTOR: u8 = 12;
pub const OP_ATOMIC_ADD_FETCH: u8 = 13;
pub const OP_ATOMIC_EXCHANGE: u8 = 14;
pub const OP_ATOMIC_COMPARE_EXCHANGE: u8 = 15;
pub const OP_VA_ARG: u8 = 16;
pub const OP_CALL: u8 = 21;
pub const OP_ASM: u8 = 24;
pub const OP_STORE: u8 = 25;
pub const OP_ATOMIC_STORE: u8 = 26;
pub const OP_VA_START: u8 = 29;
pub const OP_COPY_OBJECT: u8 = 32;
pub const OP_JUMP: u8 = 42;
pub const OP_BRANCH: u8 = 43;
pub const OP_RETURN: u8 = 46;
pub const OP_UNREACHABLE: u8 = 47;

pub const UNARY_NEG: u8 = 0;
pub const UNARY_NOT: u8 = 1;
pub const UNARY_BIT_NOT: u8 = 2;
pub const UNARY_COUNT_LEADING_ZEROS: u8 = 5;

pub const BINARY_ADD: u8 = 0;
pub const BINARY_SUB: u8 = 1;
pub const BINARY_MUL: u8 = 2;
pub const BINARY_DIV: u8 = 3;
pub const BINARY_MOD: u8 = 4;
pub const BINARY_U_DIV: u8 = 5;
pub const BINARY_U_MOD: u8 = 6;
pub const BINARY_AND: u8 = 7;
pub const BINARY_OR: u8 = 8;
pub const BINARY_XOR: u8 = 9;
pub const BINARY_SHL: u8 = 10;
pub const BINARY_SHR: u8 = 11;
pub const BINARY_U_SHR: u8 = 12;

pub const CMP_EQ: u8 = 0;
pub const CMP_NE: u8 = 1;
pub const CMP_LT: u8 = 2;
pub const CMP_LE: u8 = 3;
pub const CMP_GT: u8 = 4;
pub const CMP_GE: u8 = 5;
pub const CMP_ULT: u8 = 6;
pub const CMP_ULE: u8 = 7;
pub const CMP_UGT: u8 = 8;
pub const CMP_UGE: u8 = 9;

pub const CONVERT_S_EXT: u8 = 0;
pub const CONVERT_Z_EXT: u8 = 1;
pub const CONVERT_TRUNC: u8 = 2;
pub const CONVERT_SI_TO_FP: u8 = 3;
pub const CONVERT_UI_TO_FP: u8 = 4;
pub const CONVERT_FP_TO_SI: u8 = 5;
pub const CONVERT_FP_TO_UI: u8 = 6;
pub const CONVERT_FP_EXT: u8 = 7;
pub const CONVERT_FP_TRUNC: u8 = 8;

/// The `CppgmVectorOp` enumerators of the SSE2 operations `RustGM` emits.
pub const VECTOR_PAND: u8 = 92;
pub const VECTOR_POR: u8 = 94;
pub const VECTOR_PCMPEQB: u8 = 96;
pub const VECTOR_PCMPGTB: u8 = 100;
pub const VECTOR_PSHUFD: u8 = 183;
pub const VECTOR_PMOVMSKB: u8 = 201;
pub const VECTOR_FROMI32: u8 = 208;

pub const VALUE_TEMP: u8 = 1;
pub const VALUE_SLOT: u8 = 2;
pub const VALUE_SYMBOL: u8 = 3;
pub const VALUE_BLOCK: u8 = 4;
pub const VALUE_INTEGER: u8 = 5;
pub const VALUE_FLOAT: u8 = 6;

pub const FLAG_VOLATILE: u8 = 1;
pub const FLAG_SIGNATURE: u8 = 2;
pub const FLAG_EXACT: u8 = 16;

pub const DATA_SCALAR: u32 = 0;
pub const DATA_ADDRESS: u32 = 1;
pub const DATA_ZERO: u32 = 2;

pub const KEY_ROLE: u32 = 10;
pub const KEY_LINKAGE: u32 = 11;
pub const KEY_BINDING: u32 = 12;
pub const ROLE_ENTRY: u32 = 1;
pub const LINKAGE_DEFAULT: u32 = 0;
pub const BINDING_DEFAULT: u32 = 0;
pub const BINDING_INTERNAL: u32 = 1;
pub const BINDING_WEAK: u32 = 3;
pub const STORAGE_DEFAULT: u32 = 0;
pub const STORAGE_READONLY: u32 = 1;
pub const STORAGE_THREAD_LOCAL: u32 = 2;
pub const ARITY_VARIADIC: u8 = 1;
pub const RETURN_NORETURN: u8 = 1;

/// `CPPGM_CLASS_STATED`: the classes of a direct object are stated.
pub const CLASS_STATED: u8 = 128;
pub const CLASS_MEMORY: u8 = 16;
pub const EIGHTBYTE_NONE: u8 = 0;
pub const EIGHTBYTE_INTEGER: u8 = 1;
pub const EIGHTBYTE_SSE: u8 = 2;
pub const EIGHTBYTE_SSEUP: u8 = 3;

pub const ASM_DIALECT_INTEL: u8 = 1;
pub const ASM_MEMORY_READ_WRITE: u8 = 3;
pub const ASM_BANK_GPR: u8 = 0;
pub const ASM_BANK_VECTOR: u8 = 1;
pub const ASM_LOCATION_REGISTER: u8 = 1;
pub const ASM_OPERAND_READ: u8 = 1;
pub const ASM_PIECE_TEXT: u8 = 0;
pub const ASM_PIECE_OPERAND: u8 = 1;

/// The type word of an object of `bytes` bytes aligned to `2^align_log2`.
#[must_use]
pub fn object(bytes: u64, align_log2: u32) -> Type {
    TYPE_OBJ | (u64::from(align_log2) << 8) | (bytes << 16)
}

/// `LowirLiteral`: the 128 bits of one literal.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Literal {
    pub low: u64,
    pub high: u64,
}
/// `LowirValue`: one operand.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Value {
    pub kind: u8,
    pub float_kind: u8,
    pub reserved: u16,
    pub index: u32,
}
/// `LowirInstruction`: one instruction or terminator.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Instruction {
    pub op: u8,
    pub sub: u8,
    pub flags: u8,
    pub order: u8,
    pub result: u32,
    pub next: u32,
    /// During construction: the block the instruction belongs to.
    pub location: u32,
    pub ty: Type,
    pub ops: [Value; 3],
    pub extra_first: u32,
    pub extra_count: u32,
    pub immediate: i64,
}
/// `LowirSlot`: one stack slot.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Slot {
    pub name: u32,
    pub kind: u8,
    pub reserved: u8,
    pub index: u16,
    pub ty: Type,
}
/// `LowirBodyCounts`: the record counts of one body.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct BodyCounts {
    pub instructions: u32,
    pub operands: u32,
    pub literals: u32,
    pub slots: u32,
    pub blocks: u32,
}
/// `CppgmDataItem`: one item of a global initializer.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct DataItem {
    pub ty: Type,
    pub bits: Literal,
    pub amount: i64,
    pub symbol: u32,
    pub kind: u32,
}
/// `CppgmParameter`: one parameter of a boundary.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Parameter {
    pub ty: Type,
    pub object_bytes: u64,
    pub pass: u8,
    pub alias: u8,
    pub classes: u8,
    pub shape_element: u8,
    pub shape_count: u16,
    pub reserved: u16,
}
/// `CppgmBoundary`: the metadata of a boundary and of its result.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct Boundary {
    pub arity: u8,
    pub effects: u8,
    pub unwind: u8,
    pub returns: u8,
    pub facts: u8,
    pub result_classes: u8,
    pub result_shape_element: u8,
    pub reserved: u8,
    pub result_shape_count: u16,
    pub reserved2: u16,
}
/// `CppgmArgumentFact`: the stated classes of one call argument.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct ArgumentFact {
    pub instruction: u32,
    pub argument: u32,
    pub classes: u8,
    pub ty: u8,
    pub shape_element: u8,
    pub reserved: u8,
    pub shape_count: u16,
    pub reserved2: u16,
}
/// `CppgmAsmRegister`: one register by bank and number.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct AsmRegister {
    pub bank: u8,
    pub number: u8,
    pub bits: u16,
    pub offset: u8,
    pub reserved: [u8; 3],
}
/// `CppgmAsmGroup`: registers bound together.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct AsmGroup {
    pub first_register: u32,
    pub register_count: u16,
    pub reserved: u16,
}
/// `CppgmAsmAlternative`: one way to place an operand.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct AsmAlternative {
    pub first_group: u32,
    pub first_immediate: u32,
    pub matching: u32,
    pub group_count: u16,
    pub immediate_count: u16,
    pub locations: u8,
    pub early_write: u8,
    pub reserved: u16,
}
/// `CppgmAsmOperand`: one operand of a template.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct AsmOperand {
    pub ty: Type,
    pub input: u32,
    pub output: u32,
    pub first_alternative: u32,
    pub alternative_count: u16,
    pub flags: u8,
    pub input_storage: u8,
    pub condition: u8,
    pub reserved: [u8; 7],
}
/// `CppgmAsmPiece`: literal text or the register of one operand.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug)]
pub struct AsmPiece {
    pub kind: u8,
    pub format: u8,
    pub bits: u16,
    pub offset: u8,
    pub reserved: [u8; 3],
    pub operand: u32,
    pub first: u32,
    pub count: u32,
}

/// A record whose size differs from the pinned C layout fails to compile.
const _: [(); 16] = [(); size_of::<Literal>()];
const _: [(); 8] = [(); size_of::<Value>()];
const _: [(); 64] = [(); size_of::<Instruction>()];
const _: [(); 16] = [(); size_of::<Slot>()];
const _: [(); 20] = [(); size_of::<BodyCounts>()];
const _: [(); 40] = [(); size_of::<DataItem>()];
const _: [(); 24] = [(); size_of::<Parameter>()];
const _: [(); 12] = [(); size_of::<Boundary>()];
const _: [(); 16] = [(); size_of::<ArgumentFact>()];
const _: [(); 8] = [(); size_of::<AsmRegister>()];
const _: [(); 8] = [(); size_of::<AsmGroup>()];
const _: [(); 20] = [(); size_of::<AsmAlternative>()];
const _: [(); 32] = [(); size_of::<AsmOperand>()];
const _: [(); 20] = [(); size_of::<AsmPiece>()];
