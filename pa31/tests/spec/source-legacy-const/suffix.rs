#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(0usize)]pub fn f<const N:usize>(){}
