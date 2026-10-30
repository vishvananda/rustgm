#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
struct S;
impl S{#[rustc_has_incoherent_inherent_impls]fn wrong(&self){}}
