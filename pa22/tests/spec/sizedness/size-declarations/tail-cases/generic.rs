struct S<T: ?Sized> {
    tag: u16,
    tail: T,
}
struct U {
    tag: u8,
    tail: S<str>,
}
struct V {
    tag: u8,
    tail: S<u32>,
}
fn main() {}
