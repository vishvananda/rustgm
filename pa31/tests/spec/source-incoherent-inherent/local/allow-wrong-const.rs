#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
struct S;
impl S{#[rustc_allow_incoherent_impl]const VALUE:u8=7;}
