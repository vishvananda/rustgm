fn f<'a,'b>(p:&'a &'b u8)->&'b u8{&**p}
fn main() {}
