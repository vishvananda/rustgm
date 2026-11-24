#[lang = "bikeshed_guaranteed_no_drop"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait NoDrop {}

#[repr(C)]
pub union Packet {
    pub whole: u64,
    pub first: u8,
}

#[lang = "maybe_uninit"]
pub union Deferred {
    pub empty: (),
    pub value: u64,
}

#[lang = "manually_drop"]
#[repr(transparent)]
pub struct Hold<T: ?Sized>(pub T);

pub union Wrapped<T> {
    pub empty: (),
    pub value: Hold<T>,
}

pub union Closed {
    pub value: Hold<u64>,
}

impl Clone for u64 {
    fn clone(&self) -> Self { *self }
}

impl Copy for u64 {}

pub union Copied<T: Copy> {
    pub value: T,
}
