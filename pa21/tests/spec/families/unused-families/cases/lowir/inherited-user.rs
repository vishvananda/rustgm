trait A{}trait B:A{}trait Q{type Out<T:B>:A;}struct S;impl Q for S{type Out<T>=T;}
fn main(){}
