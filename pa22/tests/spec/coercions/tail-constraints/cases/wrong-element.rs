struct S<T: ?Sized>(T);
fn f(s: &S<[u8; 2]>) -> &S<[u16]> {
    s
}
fn main() {}
