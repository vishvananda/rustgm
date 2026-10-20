trait A<'a>{}trait Q{type Out<T:for<'a>A<'a>>:for<'a>A<'a>;}struct S;impl Q for S{type Out<U:for<'b>A<'b>>=U;}
fn main(){}
