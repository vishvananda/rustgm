#![feature(negative_impls,with_negative_coherence)]
trait A{}impl<T:?Sized>!A for &T{}trait B{}impl<T:A>B for T{}impl<T:?Sized>B for &T{}fn main(){}
