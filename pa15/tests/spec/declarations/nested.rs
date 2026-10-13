struct S<'a>(&'a u8);struct W<'a>(S<'a>);fn f<'a,'b>(_:&'a u8,x:W<'b>)->&'b u8{x.0.0}
fn main() {}
