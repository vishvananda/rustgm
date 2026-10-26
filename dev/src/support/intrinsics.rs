//! Language intrinsics carry permissions independently of their library names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Intrinsic {
    ReadVolatile,
    WriteVolatile,
    AtomicLoad,
    AtomicStore,
    AtomicSwap,
    AtomicFetchAdd,
    AtomicCompareExchange,
    AtomicCompareExchangeWeak,
    AtomicFetchSub,
    AtomicFetchAnd,
    AtomicFetchOr,
    AtomicFetchXor,
    AtomicFetchNand,
    Read,
    Write,
    AtomicFetchMin,
    AtomicFetchMax,
    ReadUnaligned,
    WriteUnaligned,
    CopyNonoverlapping,
    Copy,
    WriteBytes,
    ConstPointerAdd,
    MutPointerAdd,
    Abort,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Safety {
    Safe,
    Unsafe,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstPermission {
    Allowed,
    RuntimeOnly,
}
#[derive(Clone, Copy, Debug)]
pub struct Descriptor {
    pub safety: Safety,
    pub constant: ConstPermission,
}
impl Intrinsic {
    /// Frozen ordinary intrinsic inventory; source-only identities have no factory.
    pub const BOOTSTRAP: [Self; 24] = [
        Self::ReadVolatile,
        Self::WriteVolatile,
        Self::AtomicLoad,
        Self::AtomicStore,
        Self::AtomicSwap,
        Self::AtomicFetchAdd,
        Self::AtomicCompareExchange,
        Self::AtomicCompareExchangeWeak,
        Self::AtomicFetchSub,
        Self::AtomicFetchAnd,
        Self::AtomicFetchOr,
        Self::AtomicFetchXor,
        Self::AtomicFetchNand,
        Self::Read,
        Self::Write,
        Self::AtomicFetchMin,
        Self::AtomicFetchMax,
        Self::ReadUnaligned,
        Self::WriteUnaligned,
        Self::CopyNonoverlapping,
        Self::Copy,
        Self::WriteBytes,
        Self::ConstPointerAdd,
        Self::MutPointerAdd,
    ];
    pub const ALL: [Self; 25] = [
        Self::ReadVolatile,
        Self::WriteVolatile,
        Self::AtomicLoad,
        Self::AtomicStore,
        Self::AtomicSwap,
        Self::AtomicFetchAdd,
        Self::AtomicCompareExchange,
        Self::AtomicCompareExchangeWeak,
        Self::AtomicFetchSub,
        Self::AtomicFetchAnd,
        Self::AtomicFetchOr,
        Self::AtomicFetchXor,
        Self::AtomicFetchNand,
        Self::Read,
        Self::Write,
        Self::AtomicFetchMin,
        Self::AtomicFetchMax,
        Self::ReadUnaligned,
        Self::WriteUnaligned,
        Self::CopyNonoverlapping,
        Self::Copy,
        Self::WriteBytes,
        Self::ConstPointerAdd,
        Self::MutPointerAdd,
        Self::Abort,
    ];
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::ReadVolatile => 0,
            Self::WriteVolatile => 1,
            Self::AtomicLoad => 2,
            Self::AtomicStore => 3,
            Self::AtomicSwap => 4,
            Self::AtomicFetchAdd => 5,
            Self::AtomicCompareExchange => 6,
            Self::AtomicCompareExchangeWeak => 7,
            Self::AtomicFetchSub => 8,
            Self::AtomicFetchAnd => 9,
            Self::AtomicFetchOr => 10,
            Self::AtomicFetchXor => 11,
            Self::AtomicFetchNand => 12,
            Self::Read => 13,
            Self::Write => 14,
            Self::AtomicFetchMin => 15,
            Self::AtomicFetchMax => 16,
            Self::ReadUnaligned => 17,
            Self::WriteUnaligned => 18,
            Self::CopyNonoverlapping => 19,
            Self::Copy => 20,
            Self::WriteBytes => 21,
            Self::ConstPointerAdd => 22,
            Self::MutPointerAdd => 23,
            Self::Abort => 24,
        }
    }
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ReadVolatile => "read_volatile",
            Self::WriteVolatile => "write_volatile",
            Self::AtomicLoad => "load",
            Self::AtomicStore => "store",
            Self::AtomicSwap => "swap",
            Self::AtomicFetchAdd => "fetch_add",
            Self::AtomicCompareExchange => "compare_exchange",
            Self::AtomicCompareExchangeWeak => "compare_exchange_weak",
            Self::AtomicFetchSub => "fetch_sub",
            Self::AtomicFetchAnd => "fetch_and",
            Self::AtomicFetchOr => "fetch_or",
            Self::AtomicFetchXor => "fetch_xor",
            Self::AtomicFetchNand => "fetch_nand",
            Self::Read => "read",
            Self::Write => "write",
            Self::AtomicFetchMin => "fetch_min",
            Self::AtomicFetchMax => "fetch_max",
            Self::ReadUnaligned => "read_unaligned",
            Self::WriteUnaligned => "write_unaligned",
            Self::CopyNonoverlapping => "copy_nonoverlapping",
            Self::Copy => "copy",
            Self::WriteBytes => "write_bytes",
            Self::ConstPointerAdd | Self::MutPointerAdd => "add",
            Self::Abort => "abort",
        }
    }
    #[must_use]
    pub const fn is_volatile(self) -> bool {
        matches!(self, Self::ReadVolatile | Self::WriteVolatile)
    }
    #[must_use]
    pub const fn is_memory(self) -> bool {
        matches!(
            self,
            Self::Read
                | Self::Write
                | Self::ReadUnaligned
                | Self::WriteUnaligned
                | Self::CopyNonoverlapping
                | Self::Copy
                | Self::WriteBytes
        )
    }
    #[must_use]
    pub const fn is_counted_memory(self) -> bool {
        matches!(self, Self::CopyNonoverlapping | Self::Copy)
    }
    #[must_use]
    pub const fn is_fill(self) -> bool {
        matches!(self, Self::WriteBytes)
    }
    #[must_use]
    pub const fn is_unaligned(self) -> bool {
        matches!(self, Self::ReadUnaligned | Self::WriteUnaligned)
    }
    #[must_use]
    pub const fn is_pointer(self) -> bool {
        self.is_pointer_function() || self.is_pointer_offset()
    }
    #[must_use]
    pub const fn is_pointer_function(self) -> bool {
        self.is_volatile() || self.is_memory()
    }
    #[must_use]
    pub const fn is_pointer_offset(self) -> bool {
        matches!(self, Self::ConstPointerAdd | Self::MutPointerAdd)
    }
    #[must_use]
    pub const fn is_atomic(self) -> bool {
        matches!(
            self,
            Self::AtomicLoad
                | Self::AtomicStore
                | Self::AtomicSwap
                | Self::AtomicFetchAdd
                | Self::AtomicCompareExchange
                | Self::AtomicCompareExchangeWeak
                | Self::AtomicFetchSub
                | Self::AtomicFetchAnd
                | Self::AtomicFetchOr
                | Self::AtomicFetchXor
                | Self::AtomicFetchNand
                | Self::AtomicFetchMin
                | Self::AtomicFetchMax
        )
    }
    #[must_use]
    pub const fn writes_pointer(self) -> bool {
        matches!(
            self,
            Self::Write | Self::WriteVolatile | Self::WriteUnaligned | Self::WriteBytes
        )
    }
    #[must_use]
    pub const fn atomic_order_valid(self, order: super::atomic_ordering::Ordering) -> bool {
        match self {
            Self::AtomicLoad | Self::AtomicCompareExchange | Self::AtomicCompareExchangeWeak => {
                order.load_valid()
            }
            Self::AtomicStore => order.store_valid(),
            Self::AtomicSwap
            | Self::AtomicFetchAdd
            | Self::AtomicFetchSub
            | Self::AtomicFetchAnd
            | Self::AtomicFetchOr
            | Self::AtomicFetchXor
            | Self::AtomicFetchNand
            | Self::AtomicFetchMin
            | Self::AtomicFetchMax => true,
            Self::ReadVolatile
            | Self::WriteVolatile
            | Self::Read
            | Self::Write
            | Self::ReadUnaligned
            | Self::WriteUnaligned
            | Self::CopyNonoverlapping
            | Self::Copy
            | Self::WriteBytes
            | Self::ConstPointerAdd
            | Self::MutPointerAdd
            | Self::Abort => false,
        }
    }
    #[must_use]
    pub const fn is_bitwise(self) -> bool {
        matches!(
            self,
            Self::AtomicFetchAnd
                | Self::AtomicFetchOr
                | Self::AtomicFetchXor
                | Self::AtomicFetchNand
        )
    }
    #[must_use]
    pub const fn is_extremum(self) -> bool {
        matches!(self, Self::AtomicFetchMin | Self::AtomicFetchMax)
    }
    #[must_use]
    pub const fn uses_retry_loop(self) -> bool {
        self.is_bitwise() || self.is_extremum()
    }
    #[must_use]
    pub const fn is_compare_exchange(self) -> bool {
        matches!(
            self,
            Self::AtomicCompareExchange | Self::AtomicCompareExchangeWeak
        )
    }
    #[must_use]
    pub const fn descriptor(self) -> Descriptor {
        Descriptor {
            safety: if self.is_pointer() {
                Safety::Unsafe
            } else {
                Safety::Safe
            },
            constant: if self.is_memory() || self.is_pointer_offset() {
                ConstPermission::Allowed
            } else {
                ConstPermission::RuntimeOnly
            },
        }
    }
}
