#[lang = "freeze"]
pub unsafe auto trait Freeze {}

impl Clone for u8 {
    fn clone(&self) -> Self {
        *self
    }
}

impl Copy for u8 {}

pub trait RefOnly {}
impl<'a, T> RefOnly for &'a T {}

pub trait Value {
    type Output;
    fn value(&self) -> &Self::Output;
}

impl<'a, T> Value for &'a T {
    type Output = T;

    fn value(&self) -> &T {
        *self
    }
}

pub trait Borrowed<'a, T> {
    fn inspect(&self, value: &'a T) -> u8;
}
