struct S<'a>(&'a u8);fn f<'a,'b>(_:&'a u8,S(x):S<'b>)->&'b u8{x}
fn main() {}
