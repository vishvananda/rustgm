trait A{type Out;}trait B{}trait C{}struct S;impl A for S{type Out=u8;}impl B for u8{}fn want<T:B+C>(){}fn f<T:A<Out:B+C>>(){want::<T::Out>();}fn main(){f::<S>();}
