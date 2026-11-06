trait A{type Out;}trait B{}trait C{}struct S;impl A for S{type Out=u8;}impl B for u8{}fn f<T:A<Out=u8:B>>(){}fn main(){}
