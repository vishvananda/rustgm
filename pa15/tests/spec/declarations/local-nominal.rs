struct S<'a>(&'a u8);fn f<'a>(x:&'a u8)->&'a u8{let y:S<'a>=S(x);y.0}
fn main() {}
