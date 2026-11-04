#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
trait A{}impl A for u32{}impl !A for u32{}fn main(){}
