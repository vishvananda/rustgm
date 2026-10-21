//! Language operations identify library contracts independently of source names.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Hook {
    Neg,
    Not,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
    Index,
    IndexMut,
}
impl Hook {
    pub const ALL: [Self; 14] = [
        Self::Neg,
        Self::Not,
        Self::Add,
        Self::Sub,
        Self::Mul,
        Self::Div,
        Self::Rem,
        Self::BitAnd,
        Self::BitOr,
        Self::BitXor,
        Self::Shl,
        Self::Shr,
        Self::Index,
        Self::IndexMut,
    ];
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Neg => "Neg",
            Self::Not => "Not",
            Self::Add => "Add",
            Self::Sub => "Sub",
            Self::Mul => "Mul",
            Self::Div => "Div",
            Self::Rem => "Rem",
            Self::BitAnd => "BitAnd",
            Self::BitOr => "BitOr",
            Self::BitXor => "BitXor",
            Self::Shl => "Shl",
            Self::Shr => "Shr",
            Self::Index => "Index",
            Self::IndexMut => "IndexMut",
        }
    }
    #[must_use]
    pub const fn method(self) -> &'static str {
        match self {
            Self::Neg => "neg",
            Self::Not => "not",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Mul => "mul",
            Self::Div => "div",
            Self::Rem => "rem",
            Self::BitAnd => "bitand",
            Self::BitOr => "bitor",
            Self::BitXor => "bitxor",
            Self::Shl => "shl",
            Self::Shr => "shr",
            Self::Index => "index",
            Self::IndexMut => "index_mut",
        }
    }
    #[must_use]
    pub const fn indexing(self) -> bool {
        matches!(self, Self::Index | Self::IndexMut)
    }
    #[must_use]
    pub const fn binary(self) -> bool {
        !matches!(self, Self::Neg | Self::Not)
    }
}
