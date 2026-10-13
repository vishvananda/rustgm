type R<'a:'b,'b>=(&'a u8,&'b u8);fn f<'a,'b>(x:R<'a,'b>)->&'b u8{x.0}
fn main() {}
