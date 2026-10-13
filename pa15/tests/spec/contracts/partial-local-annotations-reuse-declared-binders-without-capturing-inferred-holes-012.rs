fn id(x:&u8)->&u8{x}fn f(x:&u8)->&u8{let p:for<'a>fn(&'a _) -> &'a _=id;p(x)}
fn main() {}
