trait Q{type Out<'a,T> where T:'a;}struct S;impl Q for S{type Out<'x,U:'x>=&'x U;}
fn main(){}
