fn g<'a:'a,'b>(_:&'a u8,x:&'b u8)->&'b u8{x}fn f<'a,'b>(s:&'static u8,x:&'b u8)->&'b u8{let mut p=g::<'a>;p=g::<'static>;p(s,x)}
fn main() {}
