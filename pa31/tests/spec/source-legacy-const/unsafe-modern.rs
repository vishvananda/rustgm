#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(0)]unsafe fn f<const N:usize>()->usize{N}
pub fn observed()->usize{unsafe{f::<7>()}}
