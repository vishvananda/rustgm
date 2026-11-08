trait Fill<R> {
    fn take<T: Fill<Self>>(_: T)
    where
        Self: Sized,
    {
    }
}
impl Fill<u8> for u16 {}
impl Fill<u8> for u8 {}
fn main() {
    <u16 as Fill<u8>>::take::<u8>(0);
}
