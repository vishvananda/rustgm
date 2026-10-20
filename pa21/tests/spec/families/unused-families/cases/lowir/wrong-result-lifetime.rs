trait Q{type Out<'a,'b>:'a;}struct S;impl Q for S{type Out<'a,'b>=&'b u8;}
fn main(){}
