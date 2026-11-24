impl Clone for u8 {
    fn clone(&self) -> Self { *self }
}
impl Copy for u8 {}

impl Clone for bool {
    fn clone(&self) -> Self { *self }
}
impl Copy for bool {}

#[lang = "add"]
pub trait Add<Rhs = Self> {
    type Output;
    fn add(self, other: Rhs) -> Self::Output;
}

impl Add for u8 {
    type Output = u8;

    fn add(self, other: u8) -> u8 { self + other }
}

#[lang = "panic_const_add_overflow"]
pub const fn addition_overflow() -> ! {
    loop {}
}
