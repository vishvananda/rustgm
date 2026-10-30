#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(0)]pub fn f<const N:usize>()->usize{N}
#[rustc_legacy_const_generics(0)]pub fn g<const N:usize>()->usize{N}
