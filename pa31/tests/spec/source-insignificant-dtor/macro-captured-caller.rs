#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{($i:item)=>{$i}}make!{#[rustc_insignificant_dtor]struct S;}
