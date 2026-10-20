trait Q{type Out<'a> where Self:'a;fn f<'a>(x:&'a Self)->Self::Out<'a>;}impl Q for u8{type Out<'a>=&'a u8;fn f<'b>(x:&'b u8)->&'b u8{x}}
fn main(){}
