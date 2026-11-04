#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A{}impl !A for u32{}fn require<T:A>(){}fn main(){require::<u32>();}
