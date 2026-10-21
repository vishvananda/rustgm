//! Language intrinsics carry permissions independently of their library names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Intrinsic {
    ReadVolatile,
    WriteVolatile,
    AtomicLoad,
    AtomicStore,
    AtomicSwap,
    AtomicFetchAdd,
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
    pub const ALL: [Self; 6] = [
        Self::ReadVolatile,
        Self::WriteVolatile,
        Self::AtomicLoad,
        Self::AtomicStore,
        Self::AtomicSwap,
        Self::AtomicFetchAdd,
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
        }
    }
    #[must_use]
    pub const fn is_volatile(self) -> bool {
        matches!(self, Self::ReadVolatile | Self::WriteVolatile)
    }
    #[must_use]
    pub const fn atomic_order_valid(self, order: super::atomic_ordering::Ordering) -> bool {
        match self {
            Self::AtomicLoad => order.load_valid(),
            Self::AtomicStore => order.store_valid(),
            Self::AtomicSwap | Self::AtomicFetchAdd => true,
            Self::ReadVolatile | Self::WriteVolatile => false,
        }
    }
    #[must_use]
    pub const fn descriptor(self) -> Descriptor {
        Descriptor {
            safety: if self.is_volatile() {
                Safety::Unsafe
            } else {
                Safety::Safe
            },
            constant: ConstPermission::RuntimeOnly,
        }
    }
}
