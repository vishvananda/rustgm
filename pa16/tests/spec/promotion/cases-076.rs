struct S<'a>(&'a u8);fn f<'a>(p:S<'a>)->&'static [S<'static>;0]{&[p;0]}
fn main() {}
