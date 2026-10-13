struct S<'a:'static>(&'a u8);type R<'a>=S<'a>;fn f<'a>(x:R<'a>)->&'static u8{x.0}
fn main() {}
