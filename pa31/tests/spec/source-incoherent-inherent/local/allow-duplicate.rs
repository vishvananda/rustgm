#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
struct S;
impl S{#[rustc_allow_incoherent_impl]#[rustc_allow_incoherent_impl]fn value(&self)->u8{7}}
