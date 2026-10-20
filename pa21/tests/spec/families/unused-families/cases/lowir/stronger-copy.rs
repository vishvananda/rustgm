trait Q{type Out<T>;}struct S;impl Q for S{type Out<T:Copy>=T;}
fn main(){}
