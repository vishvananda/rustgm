struct S(u8);fn g()->S{S(1)}fn f()->&'static [S;0]{&[g();0]}
fn main() {}
