#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
struct S{#[rustc_insignificant_dtor]value:u8}
