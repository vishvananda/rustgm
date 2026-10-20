struct S<T: ?Sized>(T);
fn f<'a>(s: &'a S<[u8; 2]>) -> &'a S<[u8]> {
    s
}
fn main() {}
