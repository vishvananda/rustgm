#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{()=>{#[rustc_insignificant_dtor]struct S;}}make!();
