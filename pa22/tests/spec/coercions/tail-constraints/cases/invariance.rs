struct S<T: ?Sized>(T);
fn f<'a, 'b: 'a>(s: &'a mut S<[&'b u8; 2]>) -> &'a mut S<[&'a u8]> {
    s
}
fn main() {}
