trait A{type Out;}trait B<'a>{}struct S;impl A for S{type Out=u8;}impl B<'static> for u8{}fn f<T:A<Out:for<'a>B<'a>>>(){}fn main(){f::<S>();}
