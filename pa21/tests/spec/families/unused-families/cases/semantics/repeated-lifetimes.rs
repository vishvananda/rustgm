trait Q{type Out<'a,'b> where 'a:'b;}struct S;impl Q for S{type Out<'x,'y>=&'y &'x u8 where 'x:'y;}
fn main(){}
