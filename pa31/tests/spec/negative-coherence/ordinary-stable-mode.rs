#![feature(negative_impls)]
trait A{}impl<T:?Sized>!A for &T{}trait B{}impl<T:A>B for T{}impl<T:?Sized>B for &T{}fn main(){}
