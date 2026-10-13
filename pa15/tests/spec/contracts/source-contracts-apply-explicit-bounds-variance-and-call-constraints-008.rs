fn id(x:&u8)->&u8{x}fn f<'a,'b>(x:&'a u8)->&'b u8{id(x)}
fn main() {}
