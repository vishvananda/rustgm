trait Q{type Out<T>:Copy;}struct S;impl Q for S{type Out<T>=T;}
fn main(){}
