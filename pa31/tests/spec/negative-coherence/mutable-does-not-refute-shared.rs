#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A{}impl<T:?Sized>!A for &mut T{}#[rustc_strict_coherence]trait B{}impl<T:A>B for T{}impl<T:?Sized>B for &T{}fn main(){}
