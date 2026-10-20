trait A<T>{}impl<T>A<T> for T{}trait Q{type Out<T,U>:A<T>;}struct S;impl Q for S{type Out<X,Y>=Y;}
fn main(){}
