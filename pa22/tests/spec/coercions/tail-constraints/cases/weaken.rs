struct S<T: ?Sized>(T);
fn f(s: &mut S<[u8; 2]>) -> &S<[u8]> {
    s
}
fn main() {}
