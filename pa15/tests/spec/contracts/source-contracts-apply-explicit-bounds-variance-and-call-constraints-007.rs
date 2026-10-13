fn id(x:&u8)->&u8{x}fn f<'a>(x:&'a u8)->&'a u8{id(x)}
fn main() {}
