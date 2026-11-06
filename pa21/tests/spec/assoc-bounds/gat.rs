trait A{type Out<T>;}trait B{}struct S;impl A for S{type Out<T>=T;}impl B for u8{}fn want<T:B>(){}fn f<T:A<Out<u8>:B>>(){want::<T::Out<u8>>();}fn main(){f::<S>();}
