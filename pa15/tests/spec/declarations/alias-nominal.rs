struct S<'a>(&'a u8);type R<'a>=S<'a>;fn f<'a>(x:R<'a>)->&'a u8{x.0}
fn main() {}
