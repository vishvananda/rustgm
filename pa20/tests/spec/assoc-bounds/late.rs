trait A{type Out;}trait B{}trait X<T:A<Out:B>>{}struct S;impl A for S{type Out=u8;}impl B for u8{}impl X<S> for u16{}fn main(){}
