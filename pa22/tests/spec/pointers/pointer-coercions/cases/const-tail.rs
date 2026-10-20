struct S<T: ?Sized> {
    tail: T,
}
const P: *const S<[u8]> = &S { tail: [7, 9] };
fn f() -> *const S<[u8]> {
    P
}
fn main() {}
