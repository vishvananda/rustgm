#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(1)]pub fn f<const N:usize>(x:&'static u32)->&'static u32{x}
