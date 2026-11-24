// Reduced from the numeric wrapper constants of the pinned Rust core.
impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}

pub struct Frame<T>(pub T);

impl Frame<u8> {
    pub const MIN: Self = Self(37);
    pub const BITS: u8 = 8;
    const HIDDEN: u8 = 19;
}

impl Frame<u16> {
    pub const MIN: Self = Self(1500);
    pub const BITS: u8 = 16;
}

pub const fn byte_min() -> u8 {
    Frame::<u8>::MIN.0
}
