struct S<'a,'b>(&'a &'b u8);fn f<'a,'b>(x:&'b u8)->S<'a,'b>{let y:&'a u8=x;loop{}}
fn main() {}
