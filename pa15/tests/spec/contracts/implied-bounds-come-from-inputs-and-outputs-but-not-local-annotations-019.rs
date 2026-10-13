enum E<'a,'b>{A(&'a &'b u8),B}fn f<'a,'b>(){let x:E<'a,'b>=E::B;}
fn main() {}
