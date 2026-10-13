fn f<'a,'b>(x:&'b u8)->&'a &'b u8{let y:&'a u8=x;loop{}}
fn main() {}
