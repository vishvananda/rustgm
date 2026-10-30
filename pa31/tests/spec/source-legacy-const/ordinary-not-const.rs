#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(0)]fn f<const N:usize>()->usize{N}
pub const V:usize=f::<7>();
