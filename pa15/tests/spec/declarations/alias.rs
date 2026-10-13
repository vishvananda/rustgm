type R<'a>=&'a u8;fn f<'a>(x:R<'a>)->R<'a>{x}
fn main() {}
