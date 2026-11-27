pub trait Iter {
    type Item;
    fn item(&self) -> Self::Item;
}

pub struct Zip<T>(T);

impl<T: Iter> Iter for Zip<T> {
    type Item = (T::Item, u8);
    fn item(&self) -> Self::Item {
        (self.0.item(), 37)
    }
}

pub trait Dispatch {
    fn read(&mut self) -> <Self as Iter>::Item where Self: Iter;
}

impl<T: Iter> Dispatch for Zip<T> {
    default fn read(&mut self) -> (T::Item, u8) {
        self.item()
    }
}
