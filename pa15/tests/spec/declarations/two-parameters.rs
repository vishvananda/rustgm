struct S<'a,'b>{x:&'a u8,y:&'b u8}fn f<'a,'b>(x:&'a u8,y:&'b u8)->S<'a,'b>{S{x,y}}
fn main() {}
