#[cfg(rustc_oracle)]
#[lang = "destruct"]
#[rustc_deny_explicit_impl]
pub const trait Destruct: PointeeSized {}

#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

pub trait Enabled {}
impl Enabled for u8 {}

pub enum Payload<T> {
    None,
    Some(T),
}

pub enum Signal<T, const N: usize> {
    Idle,
    Payload(T),
}

// Reduced from the inherent Poll<Option<T>> initializers in pinned core.
impl<T: Enabled, const N: usize> Signal<Payload<T>, N> {
    pub const WAITING: Self = Self::Idle;
    pub const STOPPED: Self = Self::Payload(Payload::None);
    pub const FLAG: u8 = 27;

    pub const fn flag() -> u8 { Self::FLAG }
}
