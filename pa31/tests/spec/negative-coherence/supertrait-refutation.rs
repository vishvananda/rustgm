#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A:B{}trait B{}impl !B for u32{}#[rustc_strict_coherence]trait C{}impl<T:A>C for T{}impl C for u32{}fn main(){}
