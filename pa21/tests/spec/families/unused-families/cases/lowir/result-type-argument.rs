trait A<T>{}impl<T>A<T> for T{}trait Q{type Out<T>:A<T>;}struct S;impl Q for S{type Out<U>=U;}
fn main(){}
