#![allow(internal_features)]
#![feature(rustc_attrs)]
mod nested{#![rustc_coherence_is_core]}
fn main(){}
