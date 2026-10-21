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
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    RemAssign,
    BitAndAssign,
    BitOrAssign,
    BitXorAssign,
    ShlAssign,
    ShrAssign,
}
impl Hook {
    pub const ALL: [Self; 24] = [
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
        Self::AddAssign,
        Self::SubAssign,
        Self::MulAssign,
        Self::DivAssign,
        Self::RemAssign,
        Self::BitAndAssign,
        Self::BitOrAssign,
        Self::BitXorAssign,
        Self::ShlAssign,
        Self::ShrAssign,
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
            Self::AddAssign => "AddAssign",
            Self::SubAssign => "SubAssign",
            Self::MulAssign => "MulAssign",
            Self::DivAssign => "DivAssign",
            Self::RemAssign => "RemAssign",
            Self::BitAndAssign => "BitAndAssign",
            Self::BitOrAssign => "BitOrAssign",
            Self::BitXorAssign => "BitXorAssign",
            Self::ShlAssign => "ShlAssign",
            Self::ShrAssign => "ShrAssign",
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
            Self::AddAssign => "add_assign",
            Self::SubAssign => "sub_assign",
            Self::MulAssign => "mul_assign",
            Self::DivAssign => "div_assign",
            Self::RemAssign => "rem_assign",
            Self::BitAndAssign => "bitand_assign",
            Self::BitOrAssign => "bitor_assign",
            Self::BitXorAssign => "bitxor_assign",
            Self::ShlAssign => "shl_assign",
            Self::ShrAssign => "shr_assign",
        }
    }
    #[must_use]
    pub const fn indexing(self) -> bool {
        matches!(self, Self::Index | Self::IndexMut)
    }
    #[must_use]
    pub const fn assignment(self) -> bool {
        matches!(
            self,
            Self::AddAssign
                | Self::SubAssign
                | Self::MulAssign
                | Self::DivAssign
                | Self::RemAssign
                | Self::BitAndAssign
                | Self::BitOrAssign
                | Self::BitXorAssign
                | Self::ShlAssign
                | Self::ShrAssign
        )
    }
    #[must_use]
    pub const fn borrowed(self) -> bool {
        self.indexing() || self.assignment()
    }
    #[must_use]
    pub const fn binary(self) -> bool {
        !matches!(self, Self::Neg | Self::Not)
    }
}
