#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(2)]pub fn f<const N:usize>(x:usize)->usize{x}
