struct S<'a>(&'a u8);fn f(x:S)->&u8{x.0}
fn main() {}
