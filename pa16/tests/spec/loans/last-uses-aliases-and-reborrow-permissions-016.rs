fn f<'a>(p:&'a u8)->&'a u8{let q=&*p;q}
fn main() {}
