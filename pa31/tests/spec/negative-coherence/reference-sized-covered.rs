#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A{}impl<T>!A for &T{}#[rustc_strict_coherence]trait B{}impl<T:A>B for T{}impl<T>B for &T{}fn main(){}
