//! Runtime operations of validated source atomic read-modify-write declarations.

#[derive(Clone, Copy)]
pub(crate) enum Update {
    Exchange,
    CompareExchange,
    Add,
    Subtract,
    And,
    Nand,
    Or,
    Xor,
    Min { signed: bool },
    Max { signed: bool },
}

impl Update {
    pub(crate) fn intrinsic(name: &str) -> Option<Self> {
        Some(match name {
            "atomic_xchg" => Self::Exchange,
            "atomic_cxchg" | "atomic_cxchgweak" => Self::CompareExchange,
            "atomic_xadd" => Self::Add,
            "atomic_xsub" => Self::Subtract,
            "atomic_and" => Self::And,
            "atomic_nand" => Self::Nand,
            "atomic_or" => Self::Or,
            "atomic_xor" => Self::Xor,
            "atomic_min" => Self::Min { signed: true },
            "atomic_umin" => Self::Min { signed: false },
            "atomic_max" => Self::Max { signed: true },
            "atomic_umax" => Self::Max { signed: false },
            _ => return None,
        })
    }

    pub(crate) const fn compares(self) -> bool {
        matches!(self, Self::CompareExchange)
    }

    pub(crate) const fn takes_operand_type(self) -> bool {
        matches!(
            self,
            Self::Add | Self::Subtract | Self::And | Self::Nand | Self::Or | Self::Xor
        )
    }
}
