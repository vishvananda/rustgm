#[lang = "freeze"]
pub unsafe auto trait Freeze {}

pub trait RefOnly {}
impl<'a, T> RefOnly for &'a T {}

pub trait Value {
    fn value(&self) -> u8;
}

impl<'a, T> Value for &'a T {
    fn value(&self) -> u8 {
        41
    }
}

pub trait Borrowed<'a, T> {
    fn inspect(&self, value: &'a T) -> u8;
}
