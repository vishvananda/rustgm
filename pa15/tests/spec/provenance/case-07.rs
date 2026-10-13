fn f<'a,'b>(p:&'a &'b mut u8)->&'a u8{&**p}
fn main() {}
