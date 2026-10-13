fn f<'a,'b>(p:&'a mut &'b mut u8)->&'a u8{&**p}
fn main() {}
