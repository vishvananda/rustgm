trait A<'a>{}trait Q{type Out<T> where for<'a>T:A<'a>;}struct S;impl Q for S{type Out<U>=U where for<'x>U:A<'x>;}
fn main(){}
