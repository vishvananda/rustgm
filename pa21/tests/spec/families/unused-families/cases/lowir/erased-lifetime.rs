trait Q{type Out<'a,'b> where 'a:'b;}struct S;impl Q for S{type Out<'a,'b>=u8;}
fn main(){}
