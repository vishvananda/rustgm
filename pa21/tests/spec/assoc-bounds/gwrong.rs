trait A{type Out<T>;}trait B{}struct S;impl A for S{type Out<T>=T;}impl B for u16{}fn f<T:A<Out<u8>:B>>(){}fn main(){f::<S>();}
