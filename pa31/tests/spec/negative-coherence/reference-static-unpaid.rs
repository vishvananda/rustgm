#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A{}impl<T:?Sized>!A for &'static T{}#[rustc_strict_coherence]trait B{}impl<T:A>B for T{}impl<'a,T:?Sized>B for &'a T{}fn main(){}
