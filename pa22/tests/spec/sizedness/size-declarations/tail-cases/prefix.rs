struct S<T: ?Sized> {
    tail: T,
}
struct U {
    bad: S<str>,
    tail: u8,
}
fn main() {}
