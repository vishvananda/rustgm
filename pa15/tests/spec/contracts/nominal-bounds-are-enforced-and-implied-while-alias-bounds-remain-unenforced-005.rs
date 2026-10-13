type R<'a:'static>=&'a u8;fn f<'a>(x:&'a u8)->R<'a>{x}
fn main() {}
