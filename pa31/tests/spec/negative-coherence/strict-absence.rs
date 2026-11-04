#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A{} #[rustc_strict_coherence]trait B{} impl<T:A>B for T{}impl B for u32{}fn main(){}
