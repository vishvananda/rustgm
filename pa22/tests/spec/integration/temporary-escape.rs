struct S<T: ?Sized> {
    tail: T,
}
fn make<'a>() -> &'a S<[u8]> {
    let s = S { tail: [3, 7] };
    &s
}
fn main() {}
