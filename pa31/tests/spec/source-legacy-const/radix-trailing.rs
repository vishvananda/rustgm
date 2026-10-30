#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(0x0,)]pub fn f<const N:usize>()->usize{N}
