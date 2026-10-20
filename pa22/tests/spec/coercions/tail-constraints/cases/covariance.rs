struct S<T: ?Sized>(T);
fn f<'a, 'b: 'a>(s: &'a S<[&'b u8; 2]>) -> &'a S<[&'a u8]> {
    s
}
fn main() {}
