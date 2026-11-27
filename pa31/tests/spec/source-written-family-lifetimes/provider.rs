#[lang = "freeze"]
pub unsafe auto trait Freeze {}

pub trait Iter {
    type Item<'a> where Self: 'a;
    fn item(&self) -> Self::Item<'_>;
}

pub struct Zip<T>(T);

impl<T: Iter> Iter for Zip<T> {
    type Item<'a> = (T::Item<'a>, u8) where Self: 'a;
    fn item(&self) -> Self::Item<'_> {
        loop {}
    }
}

pub trait Dispatch {
    fn read(&mut self) -> <Self as Iter>::Item<'_> where Self: Iter;
}

impl<T: Iter> Dispatch for Zip<T> {
    default fn read(&mut self) -> <Self as Iter>::Item<'_> {
        loop {}
    }
}

impl Iter for u8 {
    type Item<'a> = u8;
    fn item(&self) -> u8 {
        *self
    }
}

impl Dispatch for Zip<u8> {
    fn read(&mut self) -> <Self as Iter>::Item<'_> {
        loop {}
    }
}
