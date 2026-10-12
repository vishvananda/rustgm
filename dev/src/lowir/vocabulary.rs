//! The scalar subset of the pinned shared `LowIR` contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Void,
    I1,
    I8,
    U8,
    I16,
    U16,
    I32,
    U32,
    I64,
    I128,
    F32,
    F64,
    Ptr,
}
impl Type {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Void => "void",
            Self::I1 => "i1",
            Self::I8 => "i8",
            Self::U8 => "u8",
            Self::I16 => "i16",
            Self::U16 => "u16",
            Self::I32 => "i32",
            Self::U32 => "u32",
            Self::I64 => "i64",
            Self::I128 => "i128",
            Self::F32 => "f32",
            Self::F64 => "f64",
            Self::Ptr => "ptr",
        }
    }
    /// Storage width; a boolean occupies a byte containing zero or one.
    #[must_use]
    pub const fn bits(self) -> u32 {
        match self {
            Self::Void => 0,
            Self::I1 | Self::I8 | Self::U8 => 8,
            Self::I16 | Self::U16 => 16,
            Self::I32 | Self::U32 | Self::F32 => 32,
            Self::I64 | Self::F64 | Self::Ptr => 64,
            Self::I128 => 128,
        }
    }
    #[must_use]
    pub const fn is_float(self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }
    #[must_use]
    pub const fn is_integer(self) -> bool {
        !matches!(self, Self::Void | Self::F32 | Self::F64 | Self::Ptr)
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Unary {
    Neg,
    BitNot,
    Not,
    Clz,
}
impl Unary {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Neg => "neg",
            Self::BitNot => "bitnot",
            Self::Not => "not",
            Self::Clz => "clz",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Binary {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Udiv,
    Umod,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Ushr,
}
impl Binary {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Mod => "mod",
            Self::Udiv => "udiv",
            Self::Umod => "umod",
            Self::And => "and",
            Self::Or => "or",
            Self::Xor => "xor",
            Self::Shl => "shl",
            Self::Shr => "shr",
            Self::Ushr => "ushr",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Predicate {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    Ult,
    Ule,
    Ugt,
    Uge,
}
impl Predicate {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Eq => "eq",
            Self::Ne => "ne",
            Self::Lt => "lt",
            Self::Le => "le",
            Self::Gt => "gt",
            Self::Ge => "ge",
            Self::Ult => "ult",
            Self::Ule => "ule",
            Self::Ugt => "ugt",
            Self::Uge => "uge",
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Convert {
    Sext,
    Zext,
    Trunc,
    Sitofp,
    Uitofp,
    Fptosi,
    Fptoui,
    Fpext,
    Fptrunc,
}
impl Convert {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Sext => "sext",
            Self::Zext => "zext",
            Self::Trunc => "trunc",
            Self::Sitofp => "sitofp",
            Self::Uitofp => "uitofp",
            Self::Fptosi => "fptosi",
            Self::Fptoui => "fptoui",
            Self::Fpext => "fpext",
            Self::Fptrunc => "fptrunc",
        }
    }
}
