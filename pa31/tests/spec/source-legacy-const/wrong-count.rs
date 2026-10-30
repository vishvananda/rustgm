#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(0,1)]pub fn f<const N:usize>(x:usize)->usize{x}
