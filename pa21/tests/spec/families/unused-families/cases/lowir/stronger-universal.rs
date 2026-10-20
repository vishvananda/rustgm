trait A<'a>{}trait Q{type Out<T:A<'static>>;}struct S;impl Q for S{type Out<U>=U where for<'x>U:A<'x>;}
fn main(){}
