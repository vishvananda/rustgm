type R<'a,'b>=&'a u8;fn f<'a>(x:R<'a,'static>)->&u8{x}
fn main() {}
