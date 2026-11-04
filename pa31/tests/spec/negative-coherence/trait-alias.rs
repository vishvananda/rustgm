#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
#![feature(trait_alias)]
trait A{}trait B{}trait AB=A+B;impl !A for u32{}#[rustc_strict_coherence]trait C{}impl<T:AB>C for T{}impl C for u32{}fn main(){}
