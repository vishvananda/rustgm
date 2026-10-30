#![feature(rustc_attrs)]
#![allow(internal_features,dead_code)]
#[rustc_has_incoherent_inherent_impls]#[rustc_insignificant_dtor]struct S;impl S{fn read(&self)->u8{7}}
