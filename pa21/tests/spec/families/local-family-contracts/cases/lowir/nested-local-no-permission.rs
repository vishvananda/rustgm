trait Q{type Out<'a,'b> where 'a:'b;fn f()->for<'a,'b>fn(&'a u8,&'b u8)->Self::Out<'a,'b>;}impl Q for u8{type Out<'a,'b>=u8 where 'a:'b;fn f()->for<'a,'b>fn(&'a u8,&'b u8)->u8{loop{}}}
fn main(){}
