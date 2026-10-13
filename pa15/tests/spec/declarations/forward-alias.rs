type A<'a>=B<'a>;type B<'b>=&'b u8;fn f<'a>(x:A<'a>)->&'a u8{x}
fn main() {}
