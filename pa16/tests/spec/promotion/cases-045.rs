struct S(u8);fn f()->&'static fn(u8)->S{let c=S;&(c as fn(u8)->S)}
fn main() {}
