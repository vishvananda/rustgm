struct S(u8);fn f<'a>(p:S)->&'a u8{&p.0}
fn main() {}
