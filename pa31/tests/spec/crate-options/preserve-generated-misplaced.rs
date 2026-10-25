#![allow(internal_features)]
#![feature(rustc_attrs)]
macro_rules! make{()=>{#[rustc_preserve_ub_checks]fn unused(){}}}make!();
fn main(){}
