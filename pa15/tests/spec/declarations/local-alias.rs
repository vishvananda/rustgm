type R<'a>=&'a u8;fn f<'a>(x:&'a u8)->&'a u8{let y:R<'a>=x;y}
fn main() {}
