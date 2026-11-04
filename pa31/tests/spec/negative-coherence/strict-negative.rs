#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A{}impl !A for u32{}#[rustc_strict_coherence]trait B{}impl<T:A>B for T{}impl B for u32{}fn main(){}
