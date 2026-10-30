#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_insignificant_dtor]struct S{value:u8}type Alias=S;fn read(x:Alias)->u8{x.value}
