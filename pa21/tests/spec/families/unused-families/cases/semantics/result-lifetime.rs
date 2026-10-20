trait Q{type Out<'a>:'a;}struct S;impl Q for S{type Out<'a>=&'a u8;}
fn main(){}
