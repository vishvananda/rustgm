trait Fill<R> {
    fn take<T: Fill<Self>>(t: T)
    where
        Self: Sized;
}
impl Fill<u8> for u16 {
    fn take<T: Fill<u16>>(_: T) {}
}
impl Fill<u16> for u8 {
    fn take<T: Fill<u8>>(_: T) {}
}
fn main() {}
