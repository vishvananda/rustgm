#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
struct S;
#[rustc_has_incoherent_inherent_impls]
pub fn wrong(){}
