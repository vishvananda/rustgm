fn id(x:&u8)->&u8{x}fn f<'a>(){let p=id as fn(&'a u8)->&'a u8;}
fn main() {}
