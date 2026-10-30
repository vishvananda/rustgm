#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_has_incoherent_inherent_impls]
struct S;
impl S{fn value(&self)->u8{7}}
impl S{fn value(&self)->u8{9}}
