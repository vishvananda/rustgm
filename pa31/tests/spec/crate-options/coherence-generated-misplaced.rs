#![allow(internal_features)]
#![feature(rustc_attrs)]
macro_rules! make{()=>{#[rustc_coherence_is_core]fn unused(){}}}make!();
fn main(){}
