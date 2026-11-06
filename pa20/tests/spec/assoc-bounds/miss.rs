trait A{type Out;}trait B{}struct S;impl A for S{type Out=u8;}fn f<T:A<Out:B>>(){}fn main(){f::<S>();}
