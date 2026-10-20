trait Q{type Out<T:Copy>:Copy;}struct S;impl Q for S{type Out<T:Copy>=T;}
fn main(){}
