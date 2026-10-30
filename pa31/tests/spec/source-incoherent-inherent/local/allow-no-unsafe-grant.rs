#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
struct S;
impl S{#[rustc_allow_incoherent_impl]fn read(pointer:*const u8)->u8{*pointer}}
