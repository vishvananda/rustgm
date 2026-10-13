fn f<'a,'b>(p:&'a mut &'b u8)->&'b u8{&**p}
fn main() {}
