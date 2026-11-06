#![feature(trait_alias)]
trait A{type Out;}trait B{}trait C{}trait BC=B+C;struct S;impl A for S{type Out=u8;}impl B for u8{}impl C for u8{}fn want<T:B+C>(){}fn f<T:A<Out:BC>>(){want::<T::Out>();}fn main(){f::<S>();}
