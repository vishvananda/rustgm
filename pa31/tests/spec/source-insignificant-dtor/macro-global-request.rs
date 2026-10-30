#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
macro_rules! make{()=>{#[rustc_insignificant_dtor]struct S;}}make!();
