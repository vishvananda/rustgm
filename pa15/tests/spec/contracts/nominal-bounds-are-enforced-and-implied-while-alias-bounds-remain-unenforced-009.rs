struct S<'a:'static>(&'a u8);type R<'a>=S<'a>;fn f<'a>(x:&'a u8){let y:R<'a>=S(x);}
fn main() {}
