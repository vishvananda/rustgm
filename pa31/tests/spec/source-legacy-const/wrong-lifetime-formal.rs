#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(1)]pub fn f<'a,const N:usize>(x:&'a u32)->&'a u32{x}
