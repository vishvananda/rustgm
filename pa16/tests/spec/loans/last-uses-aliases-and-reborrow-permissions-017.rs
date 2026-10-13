fn f<'a>(p:&'a mut u8)->&'a mut u8{let q=&mut *p;q}
fn main() {}
