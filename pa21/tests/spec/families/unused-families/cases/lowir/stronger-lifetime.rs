trait Q{type Out<'a,'b>;}struct S;impl Q for S{type Out<'a,'b>=u8 where 'a:'b;}
fn main(){}
