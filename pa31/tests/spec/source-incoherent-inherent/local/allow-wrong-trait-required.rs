#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
struct S;
trait T{#[rustc_allow_incoherent_impl]fn value(&self)->u8;}
