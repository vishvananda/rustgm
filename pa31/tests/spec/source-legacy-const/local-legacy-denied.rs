#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(0)]fn f<const N:usize>()->usize{N}
pub fn observed()->usize{f(7)}
