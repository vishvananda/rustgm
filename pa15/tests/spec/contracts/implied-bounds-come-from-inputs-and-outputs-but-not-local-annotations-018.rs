enum E<'a,'b>{A(&'a &'b u8),B}fn f<'a,'b>(x:E<'a,'b>,y:&'b u8)->&'a u8{y}
fn main() {}
