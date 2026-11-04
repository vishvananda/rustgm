#![feature(negative_impls)]
trait A<T:Copy>{}struct N;impl !A<N> for u32{}fn main(){}
