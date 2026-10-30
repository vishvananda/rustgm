#![feature(rustc_attrs)]
#[rustc_legacy_const_generics(2,0)]pub fn f<const A:usize,const B:usize>(x:usize)->usize{x}
