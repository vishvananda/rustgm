trait Fill<R: ?Sized> {
    type Out: Fill<Self>;
}
impl Fill<u8> for u16 {
    type Out = u8;
}
impl Fill<u16> for u8 {
    type Out = u16;
}
fn main() {}
