fn f<'a,'b>(p:&'a &'b mut u8)->&'b u8{&**p}
fn main() {}
