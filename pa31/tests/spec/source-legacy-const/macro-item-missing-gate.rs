#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{($i:item)=>{$i}}
make!(#[rustc_legacy_const_generics(0)]fn f<const N:usize>()->usize{N});pub fn observed()->usize{f::<7>()}
