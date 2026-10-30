#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_insignificant_dtor]struct S(u8);fn f<'a>()->&'a S{let s=S(7);&s}
