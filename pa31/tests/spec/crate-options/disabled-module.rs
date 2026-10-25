#![allow(internal_features)]
#![feature(rustc_attrs)]
#[cfg(any())]mod disabled{#![rustc_preserve_ub_checks]}
fn main(){}
