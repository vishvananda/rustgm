#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(1)]pub fn f<const N:usize>(x:for<'a>fn(&'a u32)->&'a u32)->usize{N}
