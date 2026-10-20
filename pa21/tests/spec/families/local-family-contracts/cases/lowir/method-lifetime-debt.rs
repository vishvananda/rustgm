trait Q{type Out<'a,'b> where 'a:'b;fn f<'a:'b,'b>()->Self::Out<'a,'b>;}impl Q for u8{type Out<'a,'b>=u8 where 'a:'b;fn f<'x:'y,'y>()->u8{0}}
fn main(){}
