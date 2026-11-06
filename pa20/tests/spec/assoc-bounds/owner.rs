trait A{type Out;}trait B{}trait C{}struct S;impl A for S{type Out=u8;}impl B for u8{}trait Other{}impl Other for S{}fn f<T:A<Out:B>>(){}fn main(){f::<u8>();}
