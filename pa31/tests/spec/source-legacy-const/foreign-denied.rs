#![feature(rustc_attrs)]
unsafe extern "C"{#[rustc_legacy_const_generics(0)]pub fn f<const N:usize>();}
