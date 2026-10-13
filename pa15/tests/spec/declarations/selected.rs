struct S<'a>(&'a u8);fn f<'a,'b>(_:&'a u8,x:S<'b>)->&'b u8{x.0}
fn main() {}
