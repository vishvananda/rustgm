struct S<T: ?Sized> {
    tail: T,
}
fn f() {
    let _s: S<[u8]> = S { tail: [3, 5] };
}
fn main() {}
