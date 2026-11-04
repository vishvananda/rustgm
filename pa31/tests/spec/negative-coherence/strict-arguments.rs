#![feature(negative_impls,rustc_attrs,with_negative_coherence)]
#[rustc_strict_coherence(bad)]trait A{}fn main(){}
