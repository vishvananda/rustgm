type R<'a>=&'a u8;fn f(x:&u8)->&u8{let y:R=x;y}
fn main() {}
