#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(0)]fn f<const N:usize>()->usize{N}
pub fn observed()->usize{let a=f::<7>;a()}
