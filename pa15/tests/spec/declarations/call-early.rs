fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f(x:&u8)->&u8{id(x)}
fn main() {}
