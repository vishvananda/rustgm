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
    pub const ALL: [Self; 8] = [
        Self::ReadVolatile,
        Self::WriteVolatile,
        Self::AtomicLoad,
        Self::AtomicStore,
        Self::AtomicSwap,
        Self::AtomicFetchAdd,
        Self::AtomicCompareExchange,
        Self::AtomicCompareExchangeWeak,
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
        }
    }
    #[must_use]
    pub const fn is_volatile(self) -> bool {
        matches!(self, Self::ReadVolatile | Self::WriteVolatile)
    }
    #[must_use]
    pub const fn atomic_order_valid(self, order: super::atomic_ordering::Ordering) -> bool {
        match self {
            Self::AtomicLoad | Self::AtomicCompareExchange | Self::AtomicCompareExchangeWeak => {
                order.load_valid()
            }
            Self::AtomicStore => order.store_valid(),
            Self::AtomicSwap | Self::AtomicFetchAdd => true,
            Self::ReadVolatile | Self::WriteVolatile => false,
        }
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
            safety: if self.is_volatile() {
                Safety::Unsafe
            } else {
                Safety::Safe
            },
            constant: ConstPermission::RuntimeOnly,
        }
    }
}
