struct S<T: ?Sized>(T);
fn f(s: &S<[u8; 2]>) -> &'static S<[u8]> {
    s
}
fn main() {}
